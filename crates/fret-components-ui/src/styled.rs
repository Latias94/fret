use crate::StyleRefinement;

pub trait Stylable: Sized {
    fn apply_style(self, style: StyleRefinement) -> Self;
}

pub struct Styled<T> {
    inner: T,
    style: StyleRefinement,
}

impl<T> Styled<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            style: StyleRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = self.style.merge(style);
        self
    }

    pub fn px_1(mut self) -> Self {
        self.style = self.style.px_1();
        self
    }

    pub fn px_2(mut self) -> Self {
        self.style = self.style.px_2();
        self
    }

    pub fn px_3(mut self) -> Self {
        self.style = self.style.px_3();
        self
    }

    pub fn px_4(mut self) -> Self {
        self.style = self.style.px_4();
        self
    }

    pub fn py_1(mut self) -> Self {
        self.style = self.style.py_1();
        self
    }

    pub fn py_2(mut self) -> Self {
        self.style = self.style.py_2();
        self
    }

    pub fn py_3(mut self) -> Self {
        self.style = self.style.py_3();
        self
    }

    pub fn py_4(mut self) -> Self {
        self.style = self.style.py_4();
        self
    }

    pub fn p_1(mut self) -> Self {
        self.style = self.style.p_1();
        self
    }

    pub fn p_2(mut self) -> Self {
        self.style = self.style.p_2();
        self
    }

    pub fn p_3(mut self) -> Self {
        self.style = self.style.p_3();
        self
    }

    pub fn p_4(mut self) -> Self {
        self.style = self.style.p_4();
        self
    }

    pub fn rounded_md(mut self) -> Self {
        self.style = self.style.rounded_md();
        self
    }

    pub fn border_1(mut self) -> Self {
        self.style = self.style.border_1();
        self
    }
}

impl<T: Stylable> Styled<T> {
    pub fn finish(self) -> T {
        self.inner.apply_style(self.style)
    }
}

pub trait StyledExt: Stylable + Sized {
    fn styled(self) -> Styled<Self> {
        Styled::new(self)
    }
}

impl<T: Stylable> StyledExt for T {}

impl Stylable for crate::button::Button {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::combobox::Combobox {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::dropdown_menu::DropdownMenuButton {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::scroll_area::ScrollArea {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::select::Select {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::text_area_field::TextAreaField {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::text_field::TextField {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

impl Stylable for crate::toolbar::Toolbar {
    fn apply_style(self, style: StyleRefinement) -> Self {
        self.refine_style(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MetricRef, Space};

    #[derive(Debug, Default, Clone)]
    struct Dummy {
        style: StyleRefinement,
    }

    impl Stylable for Dummy {
        fn apply_style(mut self, style: StyleRefinement) -> Self {
            self.style = style;
            self
        }
    }

    #[test]
    fn styled_ext_builds_a_refinement_chain() {
        let dummy = Dummy::default()
            .styled()
            .px_3()
            .py_2()
            .border_1()
            .rounded_md()
            .finish();

        match dummy.style.padding_x {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N3.token_key()),
            _ => panic!("expected padding_x token"),
        }
        match dummy.style.padding_y {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N2.token_key()),
            _ => panic!("expected padding_y token"),
        }
        assert!(dummy.style.border_width.is_some());
        assert!(dummy.style.radius.is_some());
    }
}
