//! Immediate-mode bullet-list helper.

use std::sync::Arc;

use fret_core::{Corners, Px};
use fret_ui::UiHost;
use fret_ui::element::{ContainerProps, Length, MarginEdges, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme};

use super::{BulletTextOptions, UiWriterImUiFacadeExt};

const BULLET_TRACK_WIDTH: Px = Px(14.0);
const BULLET_DIAMETER: Px = Px(6.0);
const BULLET_TOP_OFFSET: Px = Px(6.0);

pub(super) fn bullet_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    text: Arc<str>,
    options: BulletTextOptions,
) {
    let element = ui.with_cx_mut(|cx| bullet_text_element(cx, text, options));
    ui.add(element);
}

fn bullet_text_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    options: BulletTextOptions,
) -> fret_ui::element::AnyElement {
    let indicator_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.indicator")));
    let label_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.label")));

    let theme = Theme::global(&*cx.app);
    let color = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));

    let mut bullet_props = ContainerProps::default();
    bullet_props.layout.size.width = Length::Px(BULLET_DIAMETER);
    bullet_props.layout.size.height = Length::Px(BULLET_DIAMETER);
    bullet_props.layout.flex.shrink = 0.0;
    bullet_props.layout.margin = MarginEdges {
        top: BULLET_TOP_OFFSET.into(),
        right: Px(0.0).into(),
        bottom: Px(0.0).into(),
        left: Px(0.0).into(),
    };
    bullet_props.background = Some(color);
    bullet_props.corner_radii = Corners::all(Px(999.0));

    let mut bullet = cx.container(bullet_props, |_cx| Vec::new());
    if let Some(test_id) = indicator_test_id {
        bullet = bullet.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let mut track_props = ContainerProps::default();
    track_props.layout.size.width = Length::Px(BULLET_TRACK_WIDTH);
    track_props.layout.size.height = Length::Auto;
    track_props.layout.flex.shrink = 0.0;
    let bullet_track = cx.container(track_props, move |_cx| vec![bullet]);

    let mut label_props = TextProps::new(text);
    label_props.layout.size.width = Length::Fill;
    label_props.layout.flex.grow = 1.0;
    label_props.layout.flex.shrink = 1.0;
    label_props.layout.flex.basis = Length::Px(Px(0.0));
    label_props.color = Some(color);
    let mut label = cx.text_props(label_props);
    if let Some(test_id) = label_test_id {
        label = label.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let row = crate::ui::h_flex(move |_cx| vec![bullet_track, label])
        .gap_metric(Px(4.0).into())
        .items(crate::Items::Start)
        .no_wrap();

    if let Some(test_id) = options.test_id {
        row.test_id(test_id).into_element(cx)
    } else {
        row.into_element(cx)
    }
}
