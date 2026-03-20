use std::sync::Arc;

use fret::app::prelude::*;
use fret::{
    children::UiElementSinkExt as _,
    icons::IconId,
    style::{LayoutRefinement, Space, Theme},
};
use fret_runtime::Model;
use fret_ui::{
    ScrollStrategy,
    element::{ContainerProps, LayoutStyle, Length, VirtualListKeyCacheMode, VirtualListOptions},
    scroll::VirtualListScrollHandle,
};

mod act {
    fret::actions!([
        RotateItems = "cookbook.virtual_list_basics.rotate.v1",
        ScrollToTarget = "cookbook.virtual_list_basics.scroll_to_target.v1",
        ScrollJump = "cookbook.virtual_list_basics.scroll_jump.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.virtual_list_basics.root";
const TEST_ID_MODE: &str = "cookbook.virtual_list_basics.mode";
const TEST_ID_MODE_MEASURED: &str = "cookbook.virtual_list_basics.mode.measured";
const TEST_ID_MODE_FIXED: &str = "cookbook.virtual_list_basics.mode.fixed";
const TEST_ID_MODE_KNOWN: &str = "cookbook.virtual_list_basics.mode.known";
const TEST_ID_TALL_ROWS: &str = "cookbook.virtual_list_basics.tall_rows";
const TEST_ID_REVERSED: &str = "cookbook.virtual_list_basics.reversed";
const TEST_ID_INDEX_KEYS: &str = "cookbook.virtual_list_basics.index_keys";
const TEST_ID_VISIBLE_ONLY_KEYS: &str = "cookbook.virtual_list_basics.visible_only_keys";
const TEST_ID_ROTATE: &str = "cookbook.virtual_list_basics.rotate";
const TEST_ID_SCROLL_TARGET: &str = "cookbook.virtual_list_basics.scroll_target";
const TEST_ID_SCROLL_JUMP_INPUT: &str = "cookbook.virtual_list_basics.scroll.jump_input";
const TEST_ID_SCROLL_JUMP_GO: &str = "cookbook.virtual_list_basics.scroll.jump_go";
const TEST_ID_LIST: &str = "cookbook.virtual_list_basics.list";
const TEST_ID_ROW_TARGET: &str = "cookbook.virtual_list_basics.row_target";

const MODE_MEASURED: &str = "measured";
const MODE_FIXED: &str = "fixed";
const MODE_KNOWN: &str = "known";

const LIST_LEN: usize = 5_000;
const TARGET_ID: u64 = 512;

#[derive(Clone)]
struct RowItem {
    id: u64,
    label: Arc<str>,
}

#[derive(Clone)]
struct VirtualListViewSettings {
    mode: Arc<str>,
    tall_rows: bool,
    reversed: bool,
    index_keys: bool,
    visible_only_keys: bool,
}

fn make_items(len: usize) -> Arc<Vec<RowItem>> {
    Arc::new(
        (0..len)
            .map(|i| RowItem {
                id: i as u64,
                label: Arc::<str>::from(format!("Row {i}")),
            })
            .collect(),
    )
}

fn row_height_at(index: usize, tall_rows: bool) -> Px {
    if tall_rows && (index % 15 == 0 || index % 17 == 0) {
        Px(56.0)
    } else {
        Px(28.0)
    }
}

struct VirtualListBasicsView {
    items: Model<Arc<Vec<RowItem>>>,
    scroll: VirtualListScrollHandle,
}

impl View for VirtualListBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            items: app.models_mut().insert(make_items(LIST_LEN)),
            scroll: VirtualListScrollHandle::new(),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();

        let mode_state = cx
            .state()
            .local_init(|| Some::<Arc<str>>(Arc::from(MODE_MEASURED)));
        let tall_rows_state = cx.state().local_init(|| false);
        let reversed_state = cx.state().local_init(|| false);
        let index_keys_state = cx.state().local_init(|| false);
        let visible_only_keys_state = cx.state().local_init(|| false);
        let jump_state = cx.state().local::<String>();

        let items = self.items.layout(cx).value_or_default();
        let len = items.len();

        let view_settings: VirtualListViewSettings = cx.data().selector_layout(
            (
                &mode_state,
                &tall_rows_state,
                &reversed_state,
                &index_keys_state,
                &visible_only_keys_state,
            ),
            |(mode, tall_rows, reversed, index_keys, visible_only_keys)| VirtualListViewSettings {
                mode: mode.unwrap_or_else(|| Arc::from(MODE_MEASURED)),
                tall_rows,
                reversed,
                index_keys,
                visible_only_keys,
            },
        );

        // Virtual lists cache `index -> key` mappings and anchor bookkeeping. If the key mapping is
        // driven by more than just the items collection (e.g. `reversed`, `index_keys`), bump the
        // effective items revision when those inputs change.
        let store = cx.app.models();
        let items_revision = store
            .revision(&self.items)
            .unwrap_or(0)
            .wrapping_add(mode_state.revision_in(store).unwrap_or(0))
            .wrapping_add(tall_rows_state.revision_in(store).unwrap_or(0))
            .wrapping_add(reversed_state.revision_in(store).unwrap_or(0))
            .wrapping_add(index_keys_state.revision_in(store).unwrap_or(0))
            .wrapping_add(visible_only_keys_state.revision_in(store).unwrap_or(0));

        let mut options = match view_settings.mode.as_ref() {
            MODE_FIXED => VirtualListOptions::fixed(Px(28.0), 10),
            MODE_KNOWN => {
                let tall = view_settings.tall_rows;
                VirtualListOptions::known(Px(28.0), 10, move |index| row_height_at(index, tall))
            }
            _ => VirtualListOptions::new(Px(28.0), 10),
        };
        options.items_revision = items_revision;
        options.gap = Px(2.0);
        if view_settings.visible_only_keys {
            options.key_cache = VirtualListKeyCacheMode::VisibleOnly;
        }

        let list_layout = LayoutStyle {
            size: fret_ui::element::SizeStyle {
                width: Length::Fill,
                height: Length::Px(Px(420.0)),
                ..Default::default()
            },
            overflow: fret_ui::element::Overflow::Clip,
            ..Default::default()
        };

        let key_at_items = Arc::clone(&items);
        let key_settings = view_settings.clone();
        let key_at = move |index: usize| {
            let mapped = if key_settings.reversed {
                len.saturating_sub(1).saturating_sub(index)
            } else {
                index
            };
            if key_settings.index_keys {
                index as fret_ui::ItemKey
            } else {
                key_at_items
                    .get(mapped)
                    .map(|it| it.id as fret_ui::ItemKey)
                    .unwrap_or(index as fret_ui::ItemKey)
            }
        };

        let row_items = Arc::clone(&items);
        let theme_for_rows = theme.clone();
        let row_settings = view_settings.clone();
        let list = cx
            .virtual_list_keyed_with_layout(
                list_layout,
                len,
                options,
                &self.scroll,
                key_at,
                move |cx, index| {
                    let mapped = if row_settings.reversed {
                        len.saturating_sub(1).saturating_sub(index)
                    } else {
                        index
                    };

                    let item = row_items.get(mapped).cloned().unwrap_or(RowItem {
                        id: mapped as u64,
                        label: Arc::<str>::from("<missing>"),
                    });

                    let zebra = (mapped % 2) == 0;
                    let background = if zebra {
                        theme_for_rows.color_token("background")
                    } else {
                        theme_for_rows.color_token("card")
                    };

                    let mut row_layout = LayoutStyle::default();
                    row_layout.size.width = Length::Fill;
                    row_layout.size.height =
                        Length::Px(row_height_at(mapped, row_settings.tall_rows));

                    let mut row = ui::container_props(
                        ContainerProps {
                            layout: row_layout,
                            background: Some(background),
                            padding: fret_core::Edges::symmetric(Px(12.0), Px(0.0)).into(),
                            border: fret_core::Edges {
                                top: Px(0.0),
                                right: Px(0.0),
                                bottom: Px(1.0),
                                left: Px(0.0),
                            },
                            border_color: Some(theme_for_rows.color_token("border")),
                            ..Default::default()
                        },
                        |_cx| {
                            [ui::h_flex(|cx| {
                                ui::children![cx;
                                    cx.text(item.label.clone()),
                                    shadcn::Badge::new(format!("#{mapped}"))
                                        .variant(shadcn::BadgeVariant::Secondary),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .w_full()
                            .h_full()]
                        },
                    )
                    .into_element(cx);

                    if item.id == TARGET_ID {
                        row = row.test_id(TEST_ID_ROW_TARGET);
                    }

                    row
                },
            )
            .test_id(TEST_ID_LIST);

        cx.actions().models::<act::RotateItems>({
            let items = self.items.clone();
            move |models| {
                models
                    .update(&items, |v| {
                        let items = Arc::make_mut(v);
                        if items.is_empty() {
                            return;
                        }
                        let by = 37 % items.len();
                        items.rotate_left(by);
                    })
                    .is_ok()
            }
        });

        cx.actions().models::<act::ScrollToTarget>({
            let items = self.items.clone();
            let reversed = reversed_state.clone_model();
            let scroll = self.scroll.clone();
            move |models| {
                let items = models
                    .read(&items, Arc::clone)
                    .ok()
                    .unwrap_or_else(|| Arc::new(Vec::new()));
                let reversed = models.read(&reversed, |v| *v).ok().unwrap_or(false);

                let pos = items.iter().position(|it| it.id == TARGET_ID).unwrap_or(0);
                let index = if reversed {
                    items.len().saturating_sub(1).saturating_sub(pos)
                } else {
                    pos
                };

                scroll.scroll_to_item(index, ScrollStrategy::Start);
                true
            }
        });

        cx.actions().models::<act::ScrollJump>({
            let jump = jump_state.clone_model();
            let scroll = self.scroll.clone();
            move |models| {
                let raw = models.read(&jump, Clone::clone).ok().unwrap_or_default();
                let index = raw.trim().parse::<usize>().ok().unwrap_or(0);
                scroll.scroll_to_item(index, ScrollStrategy::Start);
                true
            }
        });

        let mode_toggle = shadcn::ToggleGroup::single(&mode_state)
            .items([
                shadcn::ToggleGroupItem::new(MODE_MEASURED, [cx.text("Measured")])
                    .a11y_label("Measured virtualization")
                    .test_id(TEST_ID_MODE_MEASURED),
                shadcn::ToggleGroupItem::new(MODE_FIXED, [cx.text("Fixed")])
                    .a11y_label("Fixed row height virtualization")
                    .test_id(TEST_ID_MODE_FIXED),
                shadcn::ToggleGroupItem::new(MODE_KNOWN, [cx.text("Known")])
                    .a11y_label("Known variable row heights virtualization")
                    .test_id(TEST_ID_MODE_KNOWN),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
            .test_id(TEST_ID_MODE);

        let controls = ui::v_flex(|cx| {
            ui::children![cx;
                ui::h_flex(|cx| ui::children![cx; shadcn::Label::new("Measure mode:")])
                    .items_center(),
                ui::h_row(|_cx| [mode_toggle])
                    .justify_center()
                    .w_full(),
                shadcn::Separator::new(),
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Label::new("Tall rows:"),
                        shadcn::Switch::new(&tall_rows_state).test_id(TEST_ID_TALL_ROWS),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Label::new("Reversed:"),
                        shadcn::Switch::new(&reversed_state).test_id(TEST_ID_REVERSED),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Label::new("Use index keys (bad):"),
                        shadcn::Switch::new(&index_keys_state).test_id(TEST_ID_INDEX_KEYS),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Label::new("Key cache: visible only"),
                        shadcn::Switch::new(&visible_only_keys_state)
                            .test_id(TEST_ID_VISIBLE_ONLY_KEYS),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
                shadcn::Separator::new(),
                shadcn::Button::new("Rotate items (reorder)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .icon(IconId::new_static("ui.refresh"))
                    .action(act::RotateItems)
                    .test_id(TEST_ID_ROTATE),
                shadcn::Button::new(format!("Scroll to item #{TARGET_ID}"))
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .icon(IconId::new_static("ui.arrow_down"))
                    .action(act::ScrollToTarget)
                    .test_id(TEST_ID_SCROLL_TARGET),
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Input::new(&jump_state)
                            .a11y_label("Scroll to index")
                            .placeholder("Index…")
                            .test_id(TEST_ID_SCROLL_JUMP_INPUT),
                        shadcn::Button::new("Go")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .action(act::ScrollJump)
                            .test_id(TEST_ID_SCROLL_JUMP_GO),
                    ]
                })
                .gap(Space::N2),
            ]
        })
        .gap(Space::N3)
        .w_full();

        let left_settings = view_settings.clone();
        let left = ui::v_flex_build(|cx, out| {
            out.push_ui(cx, controls);
            if left_settings.index_keys {
                let alert = shadcn::Alert::new(ui::children![cx;
                    shadcn::AlertTitle::new("Index keys are intentionally wrong"),
                    shadcn::AlertDescription::new(
                        "Virtual lists must use stable keys from the model. Index identity breaks element-local state when the collection reorders.",
                    ),
                ])
                .variant(shadcn::AlertVariant::Destructive);
                out.push_ui(cx, alert);
            }
        })
        .gap(Space::N3)
        .w_full();

        let mut list_slot_layout = LayoutStyle::default();
        list_slot_layout.size.width = Length::Fill;
        list_slot_layout.flex.grow = 1.0;
        let list_slot = ui::container_props(
            ContainerProps {
                layout: list_slot_layout,
                background: Some(theme.color_token("background")),
                border: fret_core::Edges::all(Px(1.0)),
                border_color: Some(theme.color_token("border")),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            |_cx| [list],
        )
        .into_element(cx);

        let body = ui::h_flex_build(|cx, out| {
            out.push_ui(cx, left);
            out.push(list_slot);
        })
        .gap(Space::N6)
        .w_full();

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Virtual list basics"),
                        shadcn::card_description(
                            "Keyed virtualization + items_revision. Reorder the list and scroll to items without building 5,000 rows every frame.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; body]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(980.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-virtual-list-basics")
        .window("cookbook-virtual-list-basics", (1020.0, 720.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<VirtualListBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
