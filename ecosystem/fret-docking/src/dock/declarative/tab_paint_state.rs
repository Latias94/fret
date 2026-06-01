use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::super::paint::{TabChromePaintInput, TabDetailPaintInput};
use super::super::tab_overflow::TabOverflowMenuState;
use super::frame::DockSpaceElementFrame;
use super::interaction::{DeclarativeDockInteractionService, DeclarativeTabHover};

pub(super) fn declarative_tab_hover_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> DeclarativeTabHover {
    app.global::<DeclarativeDockInteractionService>()
        .map(|service| service.tab_hover(window))
        .unwrap_or_default()
}

pub(super) fn apply_declarative_tab_interaction_paint_state(
    frame: &DockSpaceElementFrame,
    hover: DeclarativeTabHover,
    menu: Option<TabOverflowMenuState>,
    tab_chrome_inputs: &mut [TabChromePaintInput],
    tab_detail_inputs: &mut [TabDetailPaintInput],
) {
    for input in tab_chrome_inputs.iter_mut() {
        input.hovered_tab = None;
    }
    for input in tab_detail_inputs.iter_mut() {
        input.hovered_tab = None;
        input.hovered_tab_close = false;
        input.hovered_tab_overflow_button = false;
        input.tab_overflow_menu = None;
    }

    if let Some((tabs, index)) = hover.tab
        && let Some(&rect) = frame.layout_all.get(&tabs)
    {
        for input in tab_chrome_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab = Some(index);
        }
        for input in tab_detail_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab = Some(index);
            input.hovered_tab_close = hover.tab_close;
        }
    }

    if let Some(tabs) = hover.overflow_button
        && let Some(&rect) = frame.layout_all.get(&tabs)
    {
        for input in tab_detail_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab_overflow_button = true;
        }
    }

    if let Some(menu) = menu
        && let Some(&rect) = frame.layout_all.get(&menu.tabs)
        && let Some(input) = tab_detail_inputs
            .iter_mut()
            .find(|input| input.rect == rect)
    {
        input.tab_overflow_menu = Some(menu);
    }
}
