use super::{WindowLogicalSize, WindowPosition};

#[derive(Debug, Clone)]
pub struct WindowCreateSpec {
    pub title: String,
    pub size: WindowLogicalSize,
    pub min_size: Option<WindowLogicalSize>,
    pub max_size: Option<WindowLogicalSize>,
    pub resize_increments: Option<WindowLogicalSize>,
    pub position: Option<WindowPosition>,
    pub visible: bool,
}

impl WindowCreateSpec {
    pub fn new(title: impl Into<String>, size: WindowLogicalSize) -> Self {
        Self {
            title: title.into(),
            size,
            min_size: None,
            max_size: None,
            resize_increments: None,
            position: None,
            visible: true,
        }
    }

    pub fn with_min_size(mut self, min_size: WindowLogicalSize) -> Self {
        self.min_size = Some(min_size);
        self
    }

    pub fn with_max_size(mut self, max_size: WindowLogicalSize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_resize_increments(mut self, resize_increments: WindowLogicalSize) -> Self {
        self.resize_increments = Some(resize_increments);
        self
    }

    pub fn with_position(mut self, position: WindowPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn normalize_size_constraints(&mut self) {
        self.min_size = self
            .min_size
            .filter(WindowLogicalSize::is_strictly_positive);
        self.max_size = self
            .max_size
            .filter(WindowLogicalSize::is_strictly_positive);
        self.resize_increments = self
            .resize_increments
            .filter(WindowLogicalSize::is_strictly_positive);

        if let (Some(min_size), Some(mut max_size)) = (self.min_size, self.max_size) {
            if max_size.width < min_size.width {
                max_size.width = min_size.width;
            }
            if max_size.height < min_size.height {
                max_size.height = min_size.height;
            }
            self.max_size = Some(max_size);
        }

        if let Some(min_size) = self.min_size {
            self.size.width = self.size.width.max(min_size.width);
            self.size.height = self.size.height.max(min_size.height);
        }

        if let Some(max_size) = self.max_size {
            self.size.width = self.size.width.min(max_size.width);
            self.size.height = self.size.height.min(max_size.height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_size_constraints_repairs_inverted_bounds_and_clamps_size() {
        let mut spec = WindowCreateSpec::new("todo", WindowLogicalSize::new(320.0, 240.0))
            .with_min_size(WindowLogicalSize::new(420.0, 560.0))
            .with_max_size(WindowLogicalSize::new(360.0, 520.0));

        spec.normalize_size_constraints();

        assert_eq!(spec.min_size, Some(WindowLogicalSize::new(420.0, 560.0)));
        assert_eq!(spec.max_size, Some(WindowLogicalSize::new(420.0, 560.0)));
        assert_eq!(spec.size, WindowLogicalSize::new(420.0, 560.0));
    }

    #[test]
    fn normalize_size_constraints_drops_non_positive_constraints_and_resize_increments() {
        let mut spec = WindowCreateSpec::new("todo", WindowLogicalSize::new(640.0, 480.0))
            .with_min_size(WindowLogicalSize::new(0.0, 320.0))
            .with_max_size(WindowLogicalSize::new(900.0, -10.0))
            .with_resize_increments(WindowLogicalSize::new(24.0, 0.0));

        spec.normalize_size_constraints();

        assert_eq!(spec.min_size, None);
        assert_eq!(spec.max_size, None);
        assert_eq!(spec.resize_increments, None);
    }
}
