use std::sync::Arc;

use fret::prelude::*;
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
    mode: Model<Option<Arc<str>>>,
    tall_rows: Model<bool>,
    reversed: Model<bool>,
    index_keys: Model<bool>,
    visible_only_keys: Model<bool>,
    jump: Model<String>,
    scroll: VirtualListScrollHandle,
}

impl View for VirtualListBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            items: app.models_mut().insert(make_items(LIST_LEN)),
            mode: app.models_mut().insert(Some(Arc::from(MODE_MEASURED))),
            tall_rows: app.models_mut().insert(false),
            reversed: app.models_mut().insert(false),
            index_keys: app.models_mut().insert(false),
            visible_only_keys: app.models_mut().insert(false),
            jump: app.models_mut().insert(String::new()),
            scroll: VirtualListScrollHandle::new(),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let items = cx
            .watch_model(&self.items)
            .layout()
            .cloned_or_else(|| Arc::new(Vec::new()));
        let len = items.len();

        let mode = self
            .mode
            .read(&mut *cx.app, |_host, v| v.clone())
            .ok()
            .flatten()
            .unwrap_or_else(|| Arc::from(MODE_MEASURED));
        let tall_rows = cx.watch_model(&self.tall_rows).layout().copied_or(false);
        let reversed = cx.watch_model(&self.reversed).layout().copied_or(false);
        let index_keys = cx.watch_model(&self.index_keys).layout().copied_or(false);
        let visible_only_keys = cx
            .watch_model(&self.visible_only_keys)
            .layout()
            .copied_or(false);

        // Virtual lists cache `index -> key` mappings and anchor bookkeeping. If the key mapping is
        // driven by more than just the items collection (e.g. `reversed`, `index_keys`), bump the
        // effective items revision when those inputs change.
        let store = cx.app.models();
        let items_revision = store
            .revision(&self.items)
            .unwrap_or(0)
            .wrapping_add(store.revision(&self.mode).unwrap_or(0))
            .wrapping_add(store.revision(&self.tall_rows).unwrap_or(0))
            .wrapping_add(store.revision(&self.reversed).unwrap_or(0))
            .wrapping_add(store.revision(&self.index_keys).unwrap_or(0))
            .wrapping_add(store.revision(&self.visible_only_keys).unwrap_or(0));

        let mut options = match mode.as_ref() {
            MODE_FIXED => VirtualListOptions::fixed(Px(28.0), 10),
            MODE_KNOWN => {
                let tall = tall_rows;
                VirtualListOptions::known(Px(28.0), 10, move |index| row_height_at(index, tall))
            }
            _ => VirtualListOptions::new(Px(28.0), 10),
        };
        options.items_revision = items_revision;
        options.gap = Px(2.0);
        if visible_only_keys {
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
        let key_at = move |index: usize| {
            let mapped = if reversed {
                len.saturating_sub(1).saturating_sub(index)
            } else {
                index
            };
            if index_keys {
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
        let list = cx
            .virtual_list_keyed_with_layout(
                list_layout,
                len,
                options,
                &self.scroll,
                key_at,
                move |cx, index| {
                    let mapped = if reversed {
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
                    row_layout.size.height = Length::Px(row_height_at(mapped, tall_rows));

                    let content = ui::h_flex(|cx| {
                        [
                            cx.text(item.label.clone()),
                            shadcn::Badge::new(format!("#{mapped}"))
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items_center()
                    .w_full()
                    .h_full()
                    .into_element(cx);

                    let mut row = cx.container(
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
                        |_cx| [content],
                    );

                    if item.id == TARGET_ID {
                        row = row.test_id(TEST_ID_ROW_TARGET);
                    }

                    row
                },
            )
            .test_id(TEST_ID_LIST);

        cx.on_action_notify_models::<act::RotateItems>({
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

        cx.on_action_notify_models::<act::ScrollToTarget>({
            let items = self.items.clone();
            let reversed = self.reversed.clone();
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

        cx.on_action_notify_models::<act::ScrollJump>({
            let jump = self.jump.clone();
            let scroll = self.scroll.clone();
            move |models| {
                let raw = models.read(&jump, Clone::clone).ok().unwrap_or_default();
                let index = raw.trim().parse::<usize>().ok().unwrap_or(0);
                scroll.scroll_to_item(index, ScrollStrategy::Start);
                true
            }
        });

        let mode_toggle = shadcn::ToggleGroup::single(self.mode.clone())
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
            .into_element(cx)
            .test_id(TEST_ID_MODE);

        let controls = ui::v_flex(|cx| {
            [
                ui::h_flex(|cx| [shadcn::Label::new("Measure mode:").into_element(cx)])
                    .items_center()
                    .into_element(cx),
                ui::h_row(|_cx| [mode_toggle])
                    .justify_center()
                    .w_full()
                    .into_element(cx),
                shadcn::Separator::new().into_element(cx),
                ui::h_flex(|cx| {
                    [
                        shadcn::Label::new("Tall rows:").into_element(cx),
                        shadcn::Switch::new(self.tall_rows.clone())
                            .test_id(TEST_ID_TALL_ROWS)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                ui::h_flex(|cx| {
                    [
                        shadcn::Label::new("Reversed:").into_element(cx),
                        shadcn::Switch::new(self.reversed.clone())
                            .test_id(TEST_ID_REVERSED)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                ui::h_flex(|cx| {
                    [
                        shadcn::Label::new("Use index keys (bad):").into_element(cx),
                        shadcn::Switch::new(self.index_keys.clone())
                            .test_id(TEST_ID_INDEX_KEYS)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                ui::h_flex(|cx| {
                    [
                        shadcn::Label::new("Key cache: visible only").into_element(cx),
                        shadcn::Switch::new(self.visible_only_keys.clone())
                            .test_id(TEST_ID_VISIBLE_ONLY_KEYS)
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                shadcn::Separator::new().into_element(cx),
                shadcn::Button::new("Rotate items (reorder)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .icon(IconId::new_static("ui.refresh"))
                    .action(act::RotateItems)
                    .into_element(cx)
                    .test_id(TEST_ID_ROTATE),
                shadcn::Button::new(format!("Scroll to item #{TARGET_ID}"))
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .icon(IconId::new_static("ui.arrow_down"))
                    .action(act::ScrollToTarget)
                    .into_element(cx)
                    .test_id(TEST_ID_SCROLL_TARGET),
                ui::h_flex(|cx| {
                    [
                        shadcn::Input::new(self.jump.clone())
                            .a11y_label("Scroll to index")
                            .placeholder("Index…")
                            .test_id(TEST_ID_SCROLL_JUMP_INPUT)
                            .into_element(cx),
                        shadcn::Button::new("Go")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .action(act::ScrollJump)
                            .into_element(cx)
                            .test_id(TEST_ID_SCROLL_JUMP_GO),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .w_full()
        .into_element(cx);

        let left = ui::v_flex_build(|cx, out| {
            out.push(controls);
            if index_keys {
                out.push(
                    shadcn::Alert::new([
                        shadcn::AlertTitle::new("Index keys are intentionally wrong")
                            .into_element(cx),
                        shadcn::AlertDescription::new(
                            "Virtual lists must use stable keys from the model. Index identity breaks element-local state when the collection reorders.",
                        )
                        .into_element(cx),
                    ])
                    .variant(shadcn::AlertVariant::Destructive)
                    .into_element(cx),
                );
            }
        })
        .gap(Space::N3)
        .w_full()
        .into_element(cx);

        let mut list_slot_layout = LayoutStyle::default();
        list_slot_layout.size.width = Length::Fill;
        list_slot_layout.flex.grow = 1.0;
        let list_slot = cx.container(
            ContainerProps {
                layout: list_slot_layout,
                background: Some(theme.color_token("background")),
                border: fret_core::Edges::all(Px(1.0)),
                border_color: Some(theme.color_token("border")),
                corner_radii: fret_core::Corners::all(Px(8.0)),
                ..Default::default()
            },
            |_cx| [list],
        );

        let body = ui::h_flex(|_cx| [left, list_slot])
            .gap(Space::N6)
            .w_full()
            .into_element(cx);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Virtual list basics").into_element(cx),
            shadcn::CardDescription::new(
                "Keyed virtualization + items_revision. Reorder the list and scroll to items without building 5,000 rows every frame.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let card = shadcn::Card::new([header, shadcn::CardContent::new([body]).into_element(cx)])
            .ui()
            .w_full()
            .max_w(Px(980.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-virtual-list-basics")
        .window("cookbook-virtual-list-basics", (1020.0, 720.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<VirtualListBasicsView>()
        .map_err(anyhow::Error::from)
}
