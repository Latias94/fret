use super::*;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutTypographyRecipe {
    TableCellGeometryLight,
    TableCellGeometryDark,
    H1GeometryLight,
    H2GeometryLight,
    H3GeometryLight,
    H4GeometryLight,
    PGeometryLight,
    LeadGeometryLight,
    MutedGeometryLight,
    LargeGeometryLight,
    BlockquoteGeometryLight,
    ListGeometryLight,
    InlineCodePaddingAndStyleLight,
    SmallTextStyleLight,
    DemoGeometrySmokeLight,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutTypographyCase {
    id: String,
    web_name: String,
    recipe: LayoutTypographyRecipe,
}

#[test]
fn web_vs_fret_layout_typography_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_typography_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutTypographyCase> =
        serde_json::from_str(raw).expect("layout typography fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout typography case={}", case.id);
        match case.recipe {
            LayoutTypographyRecipe::TableCellGeometryLight => {
                assert_eq!(case.web_name, "typography-table");
                web_vs_fret_layout_typography_table_cell_geometry_light();
            }
            LayoutTypographyRecipe::TableCellGeometryDark => {
                assert_eq!(case.web_name, "typography-table");
                web_vs_fret_layout_typography_table_cell_geometry_dark();
            }
            LayoutTypographyRecipe::H1GeometryLight => {
                assert_eq!(case.web_name, "typography-h1");
                web_vs_fret_layout_typography_h1_geometry_light();
            }
            LayoutTypographyRecipe::H2GeometryLight => {
                assert_eq!(case.web_name, "typography-h2");
                web_vs_fret_layout_typography_h2_geometry_light();
            }
            LayoutTypographyRecipe::H3GeometryLight => {
                assert_eq!(case.web_name, "typography-h3");
                web_vs_fret_layout_typography_h3_geometry_light();
            }
            LayoutTypographyRecipe::H4GeometryLight => {
                assert_eq!(case.web_name, "typography-h4");
                web_vs_fret_layout_typography_h4_geometry_light();
            }
            LayoutTypographyRecipe::PGeometryLight => {
                assert_eq!(case.web_name, "typography-p");
                web_vs_fret_layout_typography_p_geometry_light();
            }
            LayoutTypographyRecipe::LeadGeometryLight => {
                assert_eq!(case.web_name, "typography-lead");
                web_vs_fret_layout_typography_lead_geometry_light();
            }
            LayoutTypographyRecipe::MutedGeometryLight => {
                assert_eq!(case.web_name, "typography-muted");
                web_vs_fret_layout_typography_muted_geometry_light();
            }
            LayoutTypographyRecipe::LargeGeometryLight => {
                assert_eq!(case.web_name, "typography-large");
                web_vs_fret_layout_typography_large_geometry_light();
            }
            LayoutTypographyRecipe::BlockquoteGeometryLight => {
                assert_eq!(case.web_name, "typography-blockquote");
                web_vs_fret_layout_typography_blockquote_geometry_light();
            }
            LayoutTypographyRecipe::ListGeometryLight => {
                assert_eq!(case.web_name, "typography-list");
                web_vs_fret_layout_typography_list_geometry_light();
            }
            LayoutTypographyRecipe::InlineCodePaddingAndStyleLight => {
                assert_eq!(case.web_name, "typography-inline-code");
                web_vs_fret_layout_typography_inline_code_padding_and_style_light();
            }
            LayoutTypographyRecipe::SmallTextStyleLight => {
                assert_eq!(case.web_name, "typography-small-text");
                web_vs_fret_layout_typography_small_text_style_light();
            }
            LayoutTypographyRecipe::DemoGeometrySmokeLight => {
                assert_eq!(case.web_name, "typography-demo");
                web_vs_fret_layout_typography_demo_geometry_smoke_light();
            }
        }
    }
}

fn web_vs_fret_layout_typography_table_cell_geometry_light() {
    let web = read_web_golden("typography-table");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");

    let mut web_trs = Vec::new();
    web_collect_tag(web_table, "tr", &mut web_trs);
    web_trs.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_trs.len(), 4, "expected 1 header + 3 body rows");

    let web_header = web_trs[0];
    let mut web_header_cells: Vec<_> = web_header
        .children
        .iter()
        .filter(|n| n.tag == "th")
        .collect();
    web_header_cells.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_header_cells.len(), 2, "expected 2 header cells");

    let col_w0 = web_header_cells[0].rect.w;
    let col_w1 = web_header_cells[1].rect.w;

    // `border-collapse: collapse` means the cell grid is inset by half the outer border width.
    let inset = web_trs[0].rect.x;

    let mut rows: Vec<[(String, WebRect); 2]> = Vec::new();
    for (row_idx, tr) in web_trs.iter().enumerate() {
        let mut cells: Vec<_> = tr
            .children
            .iter()
            .filter(|n| n.tag == "th" || n.tag == "td")
            .collect();
        cells.sort_by(|a, b| {
            a.rect
                .x
                .partial_cmp(&b.rect.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(cells.len(), 2, "expected 2 cells in row {row_idx}");
        rows.push([
            (cells[0].text.clone().unwrap_or_default(), cells[0].rect),
            (cells[1].text.clone().unwrap_or_default(), cells[1].rect),
        ]);
    }
    let rows_ui = rows.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");
            let muted = theme.color_required("muted");

            let table = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:typography-table:table")),
                ..Default::default()
            },
            move |cx| {
                let mut table_layout = LayoutStyle::default();
                table_layout.size.width = Length::Fill;

                vec![cx.container(
                    ContainerProps {
                        layout: table_layout,
                        padding: Edges::all(Px(inset)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.column(
                            ColumnProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout
                                },
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                            },
                            move |cx| {
                                let mut out = Vec::new();
                                for (row_idx, row) in rows_ui.clone().into_iter().enumerate() {
                                    let is_header = row_idx == 0;
                                    let is_body_even = row_idx > 0 && ((row_idx - 1) % 2 == 1);

                                    let row_label = Arc::<str>::from(format!(
                                        "Golden:typography-table:row{row_idx}"
                                    ));

                                    let row_panel = cx.semantics(
                                        fret_ui::element::SemanticsProps {
                                            layout: LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Fill,
                                                    height: Length::Auto,
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            role: SemanticsRole::Panel,
                                            label: Some(row_label),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            let mut row_layout = LayoutStyle::default();
                                            row_layout.size.width = Length::Fill;

                                            let row_props = ContainerProps {
                                                layout: row_layout,
                                                padding: Edges::all(Px(0.0)),
                                                background: is_body_even.then_some(muted),
                                                shadow: None,
                                                border: Edges::all(Px(0.0)),
                                                border_color: None,
                                                corner_radii: Default::default(),
                                                ..Default::default()
                                            };

                                            vec![cx.container(row_props, move |cx| {
                                                let mut flex_layout = LayoutStyle::default();
                                                flex_layout.size.width = Length::Fill;

                                                vec![cx.row(
                                                    RowProps {
                                                        layout: flex_layout,
                                                        gap: Px(0.0),
                                                        padding: Edges::all(Px(0.0)),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Stretch,
                                                    },
                                                    move |cx| {
                                                        let mut cells_out = Vec::new();
                                                        for col_idx in 0..2 {
                                                            let label = Arc::<str>::from(format!(
                                                                "Golden:typography-table:r{row_idx}c{col_idx}"
                                                            ));
                                                            let text = row[col_idx].0.clone();
                                                            let weight = if col_idx == 0 {
                                                                col_w0
                                                            } else {
                                                                col_w1
                                                            };
                                                            let left_border = if col_idx == 0 {
                                                                1.0
                                                            } else {
                                                                0.0
                                                            };

                                                            let cell = cx.semantics(
                                                                fret_ui::element::SemanticsProps {
                                                                    layout: {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.flex.grow = weight;
                                                                        layout.flex.shrink = 1.0;
                                                                        layout.flex.basis =
                                                                            Length::Px(Px(0.0));
                                                                        layout
                                                                    },
                                                                    role: SemanticsRole::Panel,
                                                                    label: Some(label),
                                                                    ..Default::default()
                                                                },
                                                                move |cx| {
                                                                    let mut cell_layout =
                                                                        LayoutStyle::default();
                                                                    cell_layout.size.width =
                                                                        Length::Fill;

                                                                    let cell_props = ContainerProps {
                                                                        layout: cell_layout,
                                                                        padding: Edges {
                                                                            top: Px(8.0),
                                                                            right: Px(16.0),
                                                                            bottom: Px(8.0),
                                                                            left: Px(16.0),
                                                                        },
                                                                        background: None,
                                                                        shadow: None,
                                                                        border: Edges {
                                                                            top: Px(1.0),
                                                                            right: Px(1.0),
                                                                            bottom: Px(0.0),
                                                                            left: Px(left_border),
                                                                        },
                                                                        border_color: Some(border),
                                                                        corner_radii: Default::default(),
                                                                        ..Default::default()
                                                                    };

                                                                    vec![cx.container(
                                                                        cell_props,
                                                                        move |cx| {
                                                                            if is_header {
                                                                                vec![decl_text::text_prose_bold_nowrap(
                                                                                    cx,
                                                                                    text.clone(),
                                                                                )]
                                                                            } else {
                                                                                vec![decl_text::text_prose_nowrap(
                                                                                    cx,
                                                                                    text.clone(),
                                                                                )]
                                                                            }
                                                                        },
                                                                    )]
                                                                },
                                                            );
                                                            cells_out.push(cell);
                                                        }
                                                        cells_out
                                                    },
                                                )]
                                            })]
                                        },
                                    );

                                    out.push(row_panel);
                                }
                                out
                            },
                        )]
                    },
                )]
            },
        );

            vec![table]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let table = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:typography-table:table"),
    )
    .expect("fret table");
    assert_close_px(
        "typography-table table width",
        table.bounds.size.width,
        web_table.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-table table height",
        table.bounds.size.height,
        web_table.rect.h,
        1.0,
    );

    for (row_idx, web_tr) in web_trs.iter().enumerate() {
        let row = find_semantics(
            &snap,
            SemanticsRole::Panel,
            Some(&format!("Golden:typography-table:row{row_idx}")),
        )
        .unwrap_or_else(|| panic!("missing fret row {row_idx}"));

        assert_close_px(
            &format!("typography-table row[{row_idx}] y"),
            row.bounds.origin.y,
            web_tr.rect.y,
            1.0,
        );
        assert_close_px(
            &format!("typography-table row[{row_idx}] h"),
            row.bounds.size.height,
            web_tr.rect.h,
            1.0,
        );

        for col_idx in 0..2 {
            let label = format!("Golden:typography-table:r{row_idx}c{col_idx}");
            let cell = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
                .unwrap_or_else(|| panic!("missing fret cell {label}"));
            let expected = &rows[row_idx][col_idx].1;

            assert_close_px(&format!("{label} x"), cell.bounds.origin.x, expected.x, 1.0);
            assert_close_px(&format!("{label} y"), cell.bounds.origin.y, expected.y, 1.0);
            assert_close_px(
                &format!("{label} w"),
                cell.bounds.size.width,
                expected.w,
                1.0,
            );
            assert_close_px(
                &format!("{label} h"),
                cell.bounds.size.height,
                expected.h,
                1.0,
            );
        }
    }

    // Paint-backed parity: `even:bg-muted` (web uses `lab(...)`).
    let web_even_bg = web_trs[2]
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web row[2] backgroundColor");
    let expected_even_rect = web_trs[2].rect;

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) = find_scene_quad_background_with_rect_close(&scene, expected_even_rect, 2.0)
        .expect("even row background quad");
    assert_rgba_close(
        "typography-table even row background",
        color_to_rgba(bg),
        web_even_bg,
        0.02,
    );
}

fn web_vs_fret_layout_typography_table_cell_geometry_dark() {
    let web = read_web_golden("typography-table");
    let theme = web.themes.get("dark").expect("missing dark theme");

    let web_table = find_first(&theme.root, &|n| n.tag == "table").expect("web table");

    let mut web_trs = Vec::new();
    web_collect_tag(web_table, "tr", &mut web_trs);
    web_trs.sort_by(|a, b| {
        a.rect
            .y
            .partial_cmp(&b.rect.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_trs.len(), 4, "expected 1 header + 3 body rows");

    let web_header = web_trs[0];
    let mut web_header_cells: Vec<_> = web_header
        .children
        .iter()
        .filter(|n| n.tag == "th")
        .collect();
    web_header_cells.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    assert_eq!(web_header_cells.len(), 2, "expected 2 header cells");

    let col_w0 = web_header_cells[0].rect.w;
    let col_w1 = web_header_cells[1].rect.w;

    // `border-collapse: collapse` means the cell grid is inset by half the outer border width.
    let inset = web_trs[0].rect.x;

    let mut rows: Vec<[(String, WebRect); 2]> = Vec::new();
    for (row_idx, tr) in web_trs.iter().enumerate() {
        let mut cells: Vec<_> = tr
            .children
            .iter()
            .filter(|n| n.tag == "th" || n.tag == "td")
            .collect();
        cells.sort_by(|a, b| {
            a.rect
                .x
                .partial_cmp(&b.rect.x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(cells.len(), 2, "expected 2 cells in row {row_idx}");
        rows.push([
            (cells[0].text.clone().unwrap_or_default(), cells[0].rect),
            (cells[1].text.clone().unwrap_or_default(), cells[1].rect),
        ]);
    }
    let rows_ui = rows.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = theme.color_required("border");
            let muted = theme.color_required("muted");

            let table = cx.semantics(
                fret_ui::element::SemanticsProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:typography-table:table")),
                    ..Default::default()
                },
                move |cx| {
                    let mut table_layout = LayoutStyle::default();
                    table_layout.size.width = Length::Fill;

                    vec![cx.container(
                        ContainerProps {
                            layout: table_layout,
                            padding: Edges::all(Px(inset)),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.column(
                                ColumnProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout
                                    },
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                },
                                move |cx| {
                                    let mut out = Vec::new();
                                    for (row_idx, row) in rows_ui.clone().into_iter().enumerate() {
                                        let is_header = row_idx == 0;
                                        let is_body_even = row_idx > 0 && ((row_idx - 1) % 2 == 1);

                                        let row_label = Arc::<str>::from(format!(
                                            "Golden:typography-table:row{row_idx}"
                                        ));

                                        let row_panel = cx.semantics(
                                            fret_ui::element::SemanticsProps {
                                                layout: LayoutStyle {
                                                    size: SizeStyle {
                                                        width: Length::Fill,
                                                        height: Length::Auto,
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                role: SemanticsRole::Panel,
                                                label: Some(row_label),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                let mut row_layout = LayoutStyle::default();
                                                row_layout.size.width = Length::Fill;

                                                let row_props = ContainerProps {
                                                    layout: row_layout,
                                                    padding: Edges::all(Px(0.0)),
                                                    background: is_body_even.then_some(muted),
                                                    shadow: None,
                                                    border: Edges::all(Px(0.0)),
                                                    border_color: None,
                                                    corner_radii: Default::default(),
                                                    ..Default::default()
                                                };

                                                vec![cx.container(row_props, move |cx| {
                                                    let mut flex_layout = LayoutStyle::default();
                                                    flex_layout.size.width = Length::Fill;

                                                    vec![cx.row(
                                                        RowProps {
                                                            layout: flex_layout,
                                                            gap: Px(0.0),
                                                            padding: Edges::all(Px(0.0)),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Stretch,
                                                        },
                                                        move |cx| {
                                                            let mut cells_out = Vec::new();
                                                            for col_idx in 0..2 {
                                                                let label =
                                                                    Arc::<str>::from(format!(
                                                                        "Golden:typography-table:r{row_idx}c{col_idx}"
                                                                    ));
                                                                let text = row[col_idx].0.clone();
                                                                let weight = if col_idx == 0 {
                                                                    col_w0
                                                                } else {
                                                                    col_w1
                                                                };
                                                                let left_border = if col_idx == 0 {
                                                                    1.0
                                                                } else {
                                                                    0.0
                                                                };

                                                                let cell = cx.semantics(
                                                                    fret_ui::element::SemanticsProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.flex.grow = weight;
                                                                            layout.flex.shrink = 1.0;
                                                                            layout.flex.basis =
                                                                                Length::Px(Px(0.0));
                                                                            layout
                                                                        },
                                                                        role: SemanticsRole::Panel,
                                                                        label: Some(label),
                                                                        ..Default::default()
                                                                    },
                                                                    move |cx| {
                                                                        let mut cell_layout =
                                                                            LayoutStyle::default();
                                                                        cell_layout.size.width =
                                                                            Length::Fill;

                                                                        let cell_props = ContainerProps {
                                                                            layout: cell_layout,
                                                                            padding: Edges {
                                                                                top: Px(8.0),
                                                                                right: Px(16.0),
                                                                                bottom: Px(8.0),
                                                                                left: Px(16.0),
                                                                            },
                                                                            background: None,
                                                                            shadow: None,
                                                                            border: Edges {
                                                                                top: Px(1.0),
                                                                                right: Px(1.0),
                                                                                bottom: Px(0.0),
                                                                                left: Px(left_border),
                                                                            },
                                                                            border_color: Some(border),
                                                                            corner_radii: Default::default(),
                                                                            ..Default::default()
                                                                        };

                                                                        vec![cx.container(
                                                                            cell_props,
                                                                            move |cx| {
                                                                                if is_header {
                                                                                    vec![decl_text::text_prose_bold_nowrap(
                                                                                        cx,
                                                                                        text.clone(),
                                                                                    )]
                                                                                } else {
                                                                                    vec![decl_text::text_prose_nowrap(
                                                                                        cx,
                                                                                        text.clone(),
                                                                                    )]
                                                                                }
                                                                            },
                                                                        )]
                                                                    },
                                                                );
                                                                cells_out.push(cell);
                                                            }
                                                            cells_out
                                                        },
                                                    )]
                                                })]
                                            },
                                        );

                                        out.push(row_panel);
                                    }
                                    out
                                },
                            )]
                        },
                    )]
                },
            );

            vec![table]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let table = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:typography-table:table"),
    )
    .expect("fret table");
    assert_close_px(
        "typography-table table width",
        table.bounds.size.width,
        web_table.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-table table height",
        table.bounds.size.height,
        web_table.rect.h,
        1.0,
    );

    for (row_idx, web_tr) in web_trs.iter().enumerate() {
        let row = find_semantics(
            &snap,
            SemanticsRole::Panel,
            Some(&format!("Golden:typography-table:row{row_idx}")),
        )
        .unwrap_or_else(|| panic!("missing fret row {row_idx}"));

        assert_close_px(
            &format!("typography-table row[{row_idx}] y"),
            row.bounds.origin.y,
            web_tr.rect.y,
            1.0,
        );
        assert_close_px(
            &format!("typography-table row[{row_idx}] h"),
            row.bounds.size.height,
            web_tr.rect.h,
            1.0,
        );

        for col_idx in 0..2 {
            let label = format!("Golden:typography-table:r{row_idx}c{col_idx}");
            let cell = find_semantics(&snap, SemanticsRole::Panel, Some(&label))
                .unwrap_or_else(|| panic!("missing fret cell {label}"));
            let expected = &rows[row_idx][col_idx].1;

            assert_close_px(&format!("{label} x"), cell.bounds.origin.x, expected.x, 1.0);
            assert_close_px(&format!("{label} y"), cell.bounds.origin.y, expected.y, 1.0);
            assert_close_px(
                &format!("{label} w"),
                cell.bounds.size.width,
                expected.w,
                1.0,
            );
            assert_close_px(
                &format!("{label} h"),
                cell.bounds.size.height,
                expected.h,
                1.0,
            );
        }
    }

    // Paint-backed parity: `even:bg-muted` (web uses `lab(...)`).
    let web_even_bg = web_trs[2]
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web row[2] backgroundColor");
    let expected_even_rect = web_trs[2].rect;

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let (_rect, bg) = find_scene_quad_background_with_rect_close(&scene, expected_even_rect, 2.0)
        .expect("even row background quad");
    assert_rgba_close(
        "typography-table even row background",
        color_to_rgba(bg),
        web_even_bg,
        0.02,
    );
}

fn assert_prepared_text_style<'a>(
    services: &'a StyleAwareServices,
    expected_text: &str,
    expected_size: Px,
    expected_line_height: Px,
    expected_weight: u16,
) -> &'a RecordedTextPrepare {
    let record = services
        .prepared
        .iter()
        .rev()
        .find(|r| r.text == expected_text)
        .unwrap_or_else(|| {
            let mut texts: Vec<_> = services.prepared.iter().map(|r| r.text.as_str()).collect();
            texts.sort();
            panic!(
                "missing prepared text style for {expected_text:?}; seen {} prepares: {texts:?}",
                services.prepared.len()
            )
        });

    assert_eq!(record.style.size, expected_size, "text size mismatch");
    assert_eq!(
        record.style.line_height,
        Some(expected_line_height),
        "line height mismatch"
    );
    assert_eq!(
        record.style.weight.0, expected_weight,
        "font weight mismatch"
    );
    record
}

fn web_vs_fret_layout_typography_h1_geometry_light() {
    let web = read_web_golden("typography-h1");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h1 = find_first(&theme.root, &|n| n.tag == "h1").expect("web h1");

    let text = web_h1.text.clone().unwrap_or_default();
    let size = web_css_px(web_h1, "fontSize");
    let line_height = web_css_px(web_h1, "lineHeight");
    let weight = web_css_u16(web_h1, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h1")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h1 = find_by_test_id(&snap, "typography-h1");
    assert_rect_close_px("typography-h1", h1.bounds, web_h1.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_h2_geometry_light() {
    let web = read_web_golden("typography-h2");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h2 = find_first(&theme.root, &|n| n.tag == "h2").expect("web h2");

    let text = web_h2.text.clone().unwrap_or_default();
    let size = web_css_px(web_h2, "fontSize");
    let line_height = web_css_px(web_h2, "lineHeight");
    let weight = web_css_u16(web_h2, "fontWeight");
    let padding_bottom = web_css_px(web_h2, "paddingBottom");
    let border_bottom = web_css_px(web_h2, "borderBottomWidth");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border_color = theme.color_required("border");

        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        let container = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    bottom: padding_bottom,
                    ..Edges::all(Px(0.0))
                },
                border: Edges {
                    bottom: border_bottom,
                    ..Edges::all(Px(0.0))
                },
                border_color: Some(border_color),
                ..Default::default()
            },
            move |_cx| vec![heading],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h2")),
                ..Default::default()
            },
            move |_cx| vec![container],
        )]
    });

    let h2 = find_by_test_id(&snap, "typography-h2");
    assert_rect_close_px("typography-h2", h2.bounds, web_h2.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_h3_geometry_light() {
    let web = read_web_golden("typography-h3");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h3 = find_first(&theme.root, &|n| n.tag == "h3").expect("web h3");

    let text = web_h3.text.clone().unwrap_or_default();
    let size = web_css_px(web_h3, "fontSize");
    let line_height = web_css_px(web_h3, "lineHeight");
    let weight = web_css_u16(web_h3, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h3")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h3 = find_by_test_id(&snap, "typography-h3");
    assert_rect_close_px("typography-h3", h3.bounds, web_h3.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_h4_geometry_light() {
    let web = read_web_golden("typography-h4");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_h4 = find_first(&theme.root, &|n| n.tag == "h4").expect("web h4");

    let text = web_h4.text.clone().unwrap_or_default();
    let size = web_css_px(web_h4, "fontSize");
    let line_height = web_css_px(web_h4, "lineHeight");
    let weight = web_css_u16(web_h4, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let heading = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-h4")),
                ..Default::default()
            },
            move |_cx| vec![heading],
        )]
    });

    let h4 = find_by_test_id(&snap, "typography-h4");
    assert_rect_close_px("typography-h4", h4.bounds, web_h4.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_p_geometry_light() {
    let web = read_web_golden("typography-p");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-p")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-p");
    assert_rect_close_px("typography-p", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_lead_geometry_light() {
    let web = read_web_golden("typography-lead");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-lead")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-lead");
    assert_rect_close_px("typography-lead", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_muted_geometry_light() {
    let web = read_web_golden("typography-muted");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web p");

    let text = web_p.text.clone().unwrap_or_default();
    let size = web_css_px(web_p, "fontSize");
    let line_height = web_css_px(web_p, "lineHeight");
    let weight = web_css_u16(web_p, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let p = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .text_color(ColorRef::Token {
                key: "muted-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
            })
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-muted")),
                ..Default::default()
            },
            move |_cx| vec![p],
        )]
    });

    let p = find_by_test_id(&snap, "typography-muted");
    assert_rect_close_px("typography-muted", p.bounds, web_p.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_large_geometry_light() {
    let web = read_web_golden("typography-large");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_div =
        find_first(&theme.root, &|n| n.tag == "div" && n.text.is_some()).expect("web div");

    let text = web_div.text.clone().unwrap_or_default();
    let size = web_css_px(web_div, "fontSize");
    let line_height = web_css_px(web_div, "lineHeight");
    let weight = web_css_u16(web_div, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let div = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .into_element(cx);

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-large")),
                ..Default::default()
            },
            move |_cx| vec![div],
        )]
    });

    let div = find_by_test_id(&snap, "typography-large");
    assert_rect_close_px("typography-large", div.bounds, web_div.rect, 1.0);
    assert_prepared_text_style(&services, &text, size, line_height, weight);
}

fn web_vs_fret_layout_typography_blockquote_geometry_light() {
    let web = read_web_golden("typography-blockquote");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_bq = find_first(&theme.root, &|n| n.tag == "blockquote").expect("web blockquote");

    let text = web_bq.text.clone().unwrap_or_default();
    let size = web_css_px(web_bq, "fontSize");
    let line_height = web_css_px(web_bq, "lineHeight");
    let border_left = web_css_px(web_bq, "borderLeftWidth");
    let padding_left = web_css_px(web_bq, "paddingLeft");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border_color = theme.color_required("border");

        let quote = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .into_element(cx);

        let container = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    left: padding_left,
                    ..Edges::all(Px(0.0))
                },
                border: Edges {
                    left: border_left,
                    ..Edges::all(Px(0.0))
                },
                border_color: Some(border_color),
                ..Default::default()
            },
            move |_cx| vec![quote],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-blockquote")),
                ..Default::default()
            },
            move |_cx| vec![container],
        )]
    });

    let bq = find_by_test_id(&snap, "typography-blockquote");
    assert_rect_close_px("typography-blockquote", bq.bounds, web_bq.rect, 1.0);

    let record = assert_prepared_text_style(
        &services,
        &text,
        size,
        line_height,
        fret_core::FontWeight::NORMAL.0,
    );
    let _ = record;
}

fn web_vs_fret_layout_typography_list_geometry_light() {
    let web = read_web_golden("typography-list");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_ul = find_first(&theme.root, &|n| n.tag == "ul").expect("web ul");

    let mut web_lis = Vec::new();
    web_collect_tag(web_ul, "li", &mut web_lis);
    web_lis.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_lis.len(), 3, "expected 3 web li nodes");

    let li_texts: Vec<String> = web_lis
        .iter()
        .map(|li| li.text.clone().unwrap_or_default())
        .collect();

    let li_size = web_css_px(web_lis[0], "fontSize");
    let li_line_height = web_css_px(web_lis[0], "lineHeight");
    let li_weight = web_css_u16(web_lis[0], "fontWeight");

    let li_texts_for_render = li_texts.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, move |cx| {
        let mut ul_layout = LayoutStyle::default();
        ul_layout.size.width = Length::Px(Px(web_ul.rect.w));
        ul_layout.margin.left = fret_ui::element::MarginEdge::Px(Px(web_ul.rect.x));

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: ul_layout,
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-list")),
                ..Default::default()
            },
            move |cx| {
                vec![cx.column(
                    ColumnProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        gap: Px(8.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        li_texts_for_render
                            .into_iter()
                            .enumerate()
                            .map(|(idx, text)| {
                                let test_id = Arc::<str>::from(format!("typography-list-li{idx}"));
                                cx.semantics(
                                    fret_ui::element::SemanticsProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout
                                        },
                                        role: SemanticsRole::Panel,
                                        test_id: Some(test_id),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        let li = ui::text(cx, text)
                                            .text_size_px(li_size)
                                            .line_height_px(li_line_height)
                                            .font_weight(fret_core::FontWeight(li_weight))
                                            .into_element(cx);
                                        vec![li]
                                    },
                                )
                            })
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    });

    let ul = find_by_test_id(&snap, "typography-list");
    assert_rect_close_px("typography-list ul", ul.bounds, web_ul.rect, 1.0);

    for (idx, web_li) in web_lis.iter().enumerate() {
        let li = find_by_test_id(&snap, &format!("typography-list-li{idx}"));
        assert_rect_close_px(
            &format!("typography-list li[{idx}]"),
            li.bounds,
            web_li.rect,
            1.0,
        );
        assert_prepared_text_style(
            &services,
            &li_texts[idx],
            li_size,
            li_line_height,
            li_weight,
        );
    }
}

fn web_vs_fret_layout_typography_inline_code_padding_and_style_light() {
    let web = read_web_golden("typography-inline-code");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_code = find_first(&theme.root, &|n| n.tag == "code").expect("web code");

    let text = web_code.text.clone().unwrap_or_default();
    let size = web_css_px(web_code, "fontSize");
    let line_height = web_css_px(web_code, "lineHeight");
    let weight = web_css_u16(web_code, "fontWeight");
    let pt = web_css_px(web_code, "paddingTop");
    let pb = web_css_px(web_code, "paddingBottom");
    let pl = web_css_px(web_code, "paddingLeft");
    let pr = web_css_px(web_code, "paddingRight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let bg = theme.color_required("muted");

        let code_text_el = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .nowrap()
            .into_element(cx);

        let code_text = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-inline-code-text")),
                ..Default::default()
            },
            move |_cx| vec![code_text_el],
        );

        let code = cx.container(
            ContainerProps {
                padding: Edges {
                    top: pt,
                    right: pr,
                    bottom: pb,
                    left: pl,
                },
                background: Some(bg),
                corner_radii: fret_core::Corners::all(Px(4.0)),
                ..Default::default()
            },
            move |_cx| vec![code_text],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                test_id: Some(Arc::from("typography-inline-code")),
                ..Default::default()
            },
            move |_cx| vec![code],
        )]
    });

    assert_prepared_text_style(&services, &text, size, line_height, weight);

    let code = find_by_test_id(&snap, "typography-inline-code");
    let code_text = find_by_test_id(&snap, "typography-inline-code-text");

    assert_close_px(
        "inline-code padding left",
        Px(code_text.bounds.origin.x.0 - code.bounds.origin.x.0),
        pl.0,
        0.25,
    );
    assert_close_px(
        "inline-code padding top",
        Px(code_text.bounds.origin.y.0 - code.bounds.origin.y.0),
        pt.0,
        0.25,
    );

    let code_right = code.bounds.origin.x.0 + code.bounds.size.width.0;
    let text_right = code_text.bounds.origin.x.0 + code_text.bounds.size.width.0;
    assert_close_px(
        "inline-code padding right",
        Px(code_right - text_right),
        pr.0,
        0.25,
    );

    let code_bottom = code.bounds.origin.y.0 + code.bounds.size.height.0;
    let text_bottom = code_text.bounds.origin.y.0 + code_text.bounds.size.height.0;
    assert_close_px(
        "inline-code padding bottom",
        Px(code_bottom - text_bottom),
        pb.0,
        0.25,
    );
}

fn web_vs_fret_layout_typography_small_text_style_light() {
    let web = read_web_golden("typography-small");
    let theme = web.themes.get("light").expect("missing light theme");
    let web_small = find_first(&theme.root, &|n| n.tag == "small").expect("web small");

    let text = web_small.text.clone().unwrap_or_default();
    let size = web_css_px(web_small, "fontSize");
    let line_height = web_css_px(web_small, "lineHeight");
    let weight = web_css_u16(web_small, "fontWeight");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, _snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let small = ui::text(cx, text.clone())
            .text_size_px(size)
            .line_height_px(line_height)
            .font_weight(fret_core::FontWeight(weight))
            .nowrap()
            .into_element(cx);

        vec![small]
    });

    let record = assert_prepared_text_style(&services, &text, size, line_height, weight);
    assert_eq!(record.constraints.wrap, TextWrap::None);
}

fn web_vs_fret_layout_typography_demo_geometry_smoke_light() {
    let web = read_web_golden("typography-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_h1 = find_first(&theme.root, &|n| n.tag == "h1").expect("web h1");
    let web_h2 = find_first(&theme.root, &|n| n.tag == "h2").expect("web h2");
    let mut web_h3s = find_all(&theme.root, &|n| n.tag == "h3");
    web_h3s.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    let web_h3 = web_h3s.first().copied().expect("web h3");
    let web_bq = find_first(&theme.root, &|n| n.tag == "blockquote").expect("web blockquote");
    let web_ul = find_first(&theme.root, &|n| n.tag == "ul").expect("web ul");

    let mut web_lis = Vec::new();
    web_collect_tag(web_ul, "li", &mut web_lis);
    web_lis.sort_by(|a, b| a.rect.y.total_cmp(&b.rect.y));
    assert_eq!(web_lis.len(), 3, "expected 3 web li nodes");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, move |cx| {
        let h1_text = web_h1.text.clone().unwrap_or_default();
        let h2_text = web_h2.text.clone().unwrap_or_default();
        let h3_text = web_h3.text.clone().unwrap_or_default();
        let bq_text = web_bq.text.clone().unwrap_or_default();

        let h1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-h1")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, h1_text.clone())
                        .text_size_px(web_css_px(web_h1, "fontSize"))
                        .line_height_px(web_css_px(web_h1, "lineHeight"))
                        .font_weight(fret_core::FontWeight(web_css_u16(web_h1, "fontWeight")))
                        .into_element(cx),
                ]
            },
        );

        let h2 = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-h2")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let border_color = theme.color_required("border");

                let heading = ui::text(cx, h2_text.clone())
                    .text_size_px(web_css_px(web_h2, "fontSize"))
                    .line_height_px(web_css_px(web_h2, "lineHeight"))
                    .font_weight(fret_core::FontWeight(web_css_u16(web_h2, "fontWeight")))
                    .into_element(cx);

                let padding_bottom = web_css_px(web_h2, "paddingBottom");
                let border_bottom = web_css_px(web_h2, "borderBottomWidth");

                let container = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        padding: Edges {
                            bottom: padding_bottom,
                            ..Edges::all(Px(0.0))
                        },
                        border: Edges {
                            bottom: border_bottom,
                            ..Edges::all(Px(0.0))
                        },
                        border_color: Some(border_color),
                        ..Default::default()
                    },
                    move |_cx| vec![heading],
                );

                vec![container]
            },
        );

        let h3 = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-h3")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, h3_text.clone())
                        .text_size_px(web_css_px(web_h3, "fontSize"))
                        .line_height_px(web_css_px(web_h3, "lineHeight"))
                        .font_weight(fret_core::FontWeight(web_css_u16(web_h3, "fontWeight")))
                        .into_element(cx),
                ]
            },
        );

        let bq = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-blockquote")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::text(cx, bq_text.clone())
                        .text_size_px(web_css_px(web_bq, "fontSize"))
                        .line_height_px(web_css_px(web_bq, "lineHeight"))
                        .into_element(cx),
                ]
            },
        );

        let li_texts: Vec<String> = web_lis
            .iter()
            .map(|li| li.text.clone().unwrap_or_default())
            .collect();
        let li_size = web_css_px(web_lis[0], "fontSize");
        let li_line_height = web_css_px(web_lis[0], "lineHeight");

        let ul = cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("typography-demo-ul")),
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(web_ul.rect.w));
                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(web_ul.rect.x));
                    layout
                },
                role: SemanticsRole::Panel,
                ..Default::default()
            },
            move |cx| {
                vec![cx.column(
                    ColumnProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        gap: Px(8.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        li_texts
                            .into_iter()
                            .map(|t| {
                                ui::text(cx, t)
                                    .text_size_px(li_size)
                                    .line_height_px(li_line_height)
                                    .into_element(cx)
                            })
                            .collect::<Vec<_>>()
                    },
                )]
            },
        );

        vec![cx.column(
            ColumnProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
            },
            move |_cx| vec![h1, h2, bq, h3, ul],
        )]
    });

    let h1 = find_by_test_id(&snap, "typography-demo-h1");
    assert_close_px(
        "typography-demo h1 w",
        h1.bounds.size.width,
        web_h1.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo h1 h",
        h1.bounds.size.height,
        web_h1.rect.h,
        1.0,
    );

    let h2 = find_by_test_id(&snap, "typography-demo-h2");
    assert_close_px(
        "typography-demo h2 w",
        h2.bounds.size.width,
        web_h2.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo h2 h",
        h2.bounds.size.height,
        web_h2.rect.h,
        1.0,
    );

    let bq = find_by_test_id(&snap, "typography-demo-blockquote");
    assert_close_px(
        "typography-demo blockquote w",
        bq.bounds.size.width,
        web_bq.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo blockquote h",
        bq.bounds.size.height,
        web_bq.rect.h,
        1.0,
    );

    let h3 = find_by_test_id(&snap, "typography-demo-h3");
    assert_close_px(
        "typography-demo h3 w",
        h3.bounds.size.width,
        web_h3.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo h3 h",
        h3.bounds.size.height,
        web_h3.rect.h,
        1.0,
    );

    let ul = find_by_test_id(&snap, "typography-demo-ul");
    assert_close_px(
        "typography-demo ul x",
        ul.bounds.origin.x,
        web_ul.rect.x,
        1.0,
    );
    assert_close_px(
        "typography-demo ul w",
        ul.bounds.size.width,
        web_ul.rect.w,
        1.0,
    );
    assert_close_px(
        "typography-demo ul h",
        ul.bounds.size.height,
        web_ul.rect.h,
        1.0,
    );
}
