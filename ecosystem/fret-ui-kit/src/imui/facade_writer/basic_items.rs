use super::*;

pub(super) fn text<H, W>(ui: &mut W, text: Arc<str>)
where
    H: UiHost,
    W: UiWriter<H> + ?Sized,
{
    let element =
        ui.with_cx_mut(|cx| crate::declarative::text::text_section_chrome_label(cx, text));
    ui.add(element);
}

pub(super) fn text_wrapped<H, W>(ui: &mut W, text: Arc<str>)
where
    H: UiHost,
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| crate::declarative::text::text_compact_paragraph(cx, text));
    ui.add(element);
}

pub(super) fn bullet_text_with_options<H, W>(ui: &mut W, text: Arc<str>, options: BulletTextOptions)
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    bullet_text_controls::bullet_text_with_options(ui, text, options);
}

pub(super) fn debug_draw_with_options<H, W, K, F>(
    ui: &mut W,
    id: K,
    options: DebugDrawOptions,
    draw: F,
) -> DebugDrawResponse
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    K: Hash,
    F: FnOnce(&mut ImUiDebugDrawList),
{
    debug_draw_controls::debug_draw_with_options(ui, id, options, draw)
}

pub(super) fn separator<H, W>(ui: &mut W)
where
    H: UiHost,
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| {
        let mut props = fret_ui::element::ContainerProps::default();
        let theme = fret_ui::Theme::global(&*cx.app);
        props.background = Some(theme.color_token("border"));
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.layout.size.height = fret_ui::element::Length::Px(fret_core::Px(1.0));
        cx.container(props, |_| Vec::new())
    });
    ui.add(element);
}

pub(super) fn separator_text_with_options<H, W>(
    ui: &mut W,
    label: Arc<str>,
    options: SeparatorTextOptions,
) where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    separator_text_controls::separator_text_with_options(ui, label, options);
}
