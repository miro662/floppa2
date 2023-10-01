use cgmath::Vector2;

use super::texture_ref::TextureRef;

/// Describes a sprite - something that can be rendered on screen
pub struct Sprite {
    pub(super) texture: TextureRef,
    pub(super) size: Vector2<u32>,
}

impl Sprite {
    /// Returns sprite's size
    pub fn size(&self) -> Vector2<u32> {
        self.size
    }
}
