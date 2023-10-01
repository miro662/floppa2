use std::path::Path;

use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};

use crate::renderer::{sprite::Sprite, Renderer, TextureData};
pub struct Image(DynamicImage);

impl Image {
    pub fn load_from_file(path: impl AsRef<Path>) -> Image {
        let image = ImageReader::open(path).unwrap().decode().unwrap();
        Image(image)
    }
}

impl TextureData for Image {
    fn data(&self) -> Vec<u8> {
        self.0.to_rgba8().to_vec()
    }

    fn size(&self) -> cgmath::Vector2<u32> {
        self.0.dimensions().into()
    }
}

/// Additional utility methods for renderer
pub trait RendererExt {
    /// Loads sprite from image file
    fn create_sprite_from_file(&self, path: impl AsRef<Path>) -> Sprite;
}

impl RendererExt for Renderer {
    fn create_sprite_from_file(&self, path: impl AsRef<Path>) -> Sprite {
        let image = Image::load_from_file(path);
        self.create_sprite(image)
    }
}
