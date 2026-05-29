macro_rules! image_button_surface_methods {
    () => {
        fn image_item(&mut self, id: &str, image: fret_core::ImageId, size: Size) -> ResponseExt {
            self.image_item_with_options(id, image, size, ImageItemOptions::default())
        }

        fn image_item_with_options(
            &mut self,
            id: &str,
            image: fret_core::ImageId,
            size: Size,
            options: ImageItemOptions,
        ) -> ResponseExt {
            image_items::image_item_with_options(self, id, image, size, options)
        }

        fn image_button(&mut self, id: &str, image: fret_core::ImageId, size: Size) -> ResponseExt {
            self.image_button_with_options(id, image, size, ImageItemOptions::button())
        }

        fn image_button_with_options(
            &mut self,
            id: &str,
            image: fret_core::ImageId,
            size: Size,
            options: ImageItemOptions,
        ) -> ResponseExt {
            image_items::image_button_with_options(self, id, image, size, options)
        }
    };
}

pub(crate) use image_button_surface_methods;
