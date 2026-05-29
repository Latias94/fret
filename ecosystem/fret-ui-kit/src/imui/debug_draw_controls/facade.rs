use std::hash::Hash;
use std::sync::Arc;

use fret_ui::UiHost;

use super::super::UiWriterImUiFacadeExt;
use super::{
    DebugDrawCommand, DebugDrawOptions, DebugDrawResponse, ImUiDebugDrawList,
    element::debug_draw_element,
};

pub(in crate::imui) fn debug_draw_with_options<H, W, K, F>(
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
    let mut list = ImUiDebugDrawList::default();
    draw(&mut list);
    list.channels_merge();
    let list_summary = list.list_summary();
    let command_summaries = Arc::from(list.command_summaries().into_boxed_slice());
    let commands: Arc<[DebugDrawCommand]> = Arc::from(list.commands.into_boxed_slice());
    let mut response = super::ResponseExt::default();
    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        cx.keyed(("fret-ui-kit.imui.debug_draw", id), |cx| {
            debug_draw_element(cx, commands, options, response)
        })
    });
    ui.add(element);
    DebugDrawResponse::new(response, list_summary, command_summaries)
}
