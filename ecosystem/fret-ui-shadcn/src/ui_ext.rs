use crate::button::Button;
use crate::checkbox::Checkbox;
use crate::input::Input;
use crate::popover::PopoverContent;
use crate::textarea::Textarea;
use crate::tooltip::TooltipContent;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, UiIntoElement, UiPatchTarget};

impl UiPatchTarget for Button {
    fn apply_chrome(self, chrome: ChromeRefinement) -> Self {
        self.refine_style(chrome)
    }

    fn apply_layout(self, layout: LayoutRefinement) -> Self {
        self.refine_layout(layout)
    }
}

impl UiIntoElement for Button {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Button::into_element(self, cx)
    }
}

impl UiPatchTarget for PopoverContent {
    fn apply_chrome(self, chrome: ChromeRefinement) -> Self {
        self.refine_style(chrome)
    }

    fn apply_layout(self, layout: LayoutRefinement) -> Self {
        self.refine_layout(layout)
    }
}

impl UiIntoElement for PopoverContent {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PopoverContent::into_element(self, cx)
    }
}

impl UiPatchTarget for TooltipContent {
    fn apply_chrome(self, chrome: ChromeRefinement) -> Self {
        self.refine_style(chrome)
    }

    fn apply_layout(self, layout: LayoutRefinement) -> Self {
        self.refine_layout(layout)
    }
}

impl UiIntoElement for TooltipContent {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        TooltipContent::into_element(self, cx)
    }
}

impl UiPatchTarget for Input {
    fn apply_chrome(self, chrome: ChromeRefinement) -> Self {
        self.refine_style(chrome)
    }

    fn apply_layout(self, layout: LayoutRefinement) -> Self {
        self.refine_layout(layout)
    }
}

impl UiIntoElement for Input {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Input::into_element(self, cx)
    }
}

impl UiPatchTarget for Checkbox {
    fn apply_chrome(self, chrome: ChromeRefinement) -> Self {
        self.refine_style(chrome)
    }

    fn apply_layout(self, layout: LayoutRefinement) -> Self {
        self.refine_layout(layout)
    }
}

impl UiIntoElement for Checkbox {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Checkbox::into_element(self, cx)
    }
}

impl UiPatchTarget for Textarea {
    fn apply_chrome(self, chrome: ChromeRefinement) -> Self {
        self.refine_style(chrome)
    }

    fn apply_layout(self, layout: LayoutRefinement) -> Self {
        self.refine_layout(layout)
    }
}

impl UiIntoElement for Textarea {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Textarea::into_element(self, cx)
    }
}
