mod render_thread;
pub mod sprite;
mod texture_ref;

use std::{
    sync::mpsc::{self},
    thread, vec,
};

use cgmath::Vector2;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use self::{
    render_thread::{RenderThreadMessage, RendererThread},
    sprite::Sprite,
    texture_ref::{TextureRef, TextureRefManager},
};

/// Trait which describes windows that can be used with renderer
pub trait CompatibleWindow: HasRawWindowHandle + HasRawDisplayHandle {}
impl<T: HasRawWindowHandle + HasRawDisplayHandle> CompatibleWindow for T {}

/// An RGBA color
pub type Color = wgpu::Color;

/// Allows rendering 2D pixel-perfect graphics on a compatible window
pub struct Renderer {
    renderer_thread_tx: mpsc::Sender<RenderThreadMessage>,
    texture_ref_manager: TextureRefManager,
}

impl Renderer {
    /// Creates renderer compatible with provided window, for given size
    pub fn compatible_with<T>(window: impl CompatibleWindow, size: (u32, u32)) -> Renderer {
        let renderer_thread = RendererThread::compatible_with(window, size);
        let (tx, rx) = mpsc::channel();
        thread::spawn(|| renderer_thread.run(rx));
        let texture_ref_manager = TextureRefManager::new();
        Renderer {
            renderer_thread_tx: tx,
            texture_ref_manager,
        }
    }

    /// Notifies renderer about window's resize
    pub fn resize(&self, size: Vector2<u32>) {
        self.renderer_thread_tx
            .send(RenderThreadMessage::Resize(size))
            .unwrap();
    }

    /// Render things described by a callback to a window
    pub fn render(&self, callback: impl FnOnce(&mut RenderCommands)) {
        let mut target = RenderCommands {
            clear_color: Some(Color::BLACK),
            blits: vec![],
        };
        callback(&mut target);
        self.renderer_thread_tx
            .send(RenderThreadMessage::Render(target))
            .unwrap();
    }

    /// Creates a sprite, loading data provided in a param to it
    pub fn create_sprite(&self, data: impl TextureData) -> Sprite {
        let texture_ref = self.texture_ref_manager.next();
        self.renderer_thread_tx
            .send(RenderThreadMessage::LoadTexture(
                texture_ref,
                data.data(),
                data.size(),
            ))
            .unwrap();
        Sprite {
            texture: texture_ref,
            size: data.size(),
        }
    }
}

/// Describes a single blit (sprite drawing) operation
#[derive(Debug, Clone)]
pub struct BlitCommand {
    pub(crate) texture_id: TextureRef,
    pub(crate) position: Vector2<u32>,
    pub(crate) size: Vector2<u32>,
    pub(crate) color: Color,
}

impl BlitCommand {
    /// Moves a sprite to a given screen position
    pub fn at(&mut self, position: impl Into<Vector2<u32>>) -> &mut Self {
        self.position = position.into();
        self
    }

    /// Changes sprite's color to a given one
    pub fn with_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}

/// Allows to define render operations
#[derive(Debug)]
pub struct RenderCommands {
    clear_color: Option<Color>,
    blits: Vec<BlitCommand>,
}

impl RenderCommands {
    /// Changes color that is used to clear target before rendering
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = Some(color);
    }

    /// Draws given sprite
    pub fn draw(&mut self, sprite: &Sprite) -> &mut BlitCommand {
        let blit_command = BlitCommand {
            texture_id: sprite.texture,
            position: (0, 0).into(),
            size: sprite.size,
            color: Color::WHITE,
        };
        self.blits.push(blit_command);
        self.blits
            .last_mut()
            .expect("Should be inserted by last command")
    }
}

impl Default for RenderCommands {
    fn default() -> Self {
        Self {
            clear_color: Some(Color::BLACK),
            blits: vec![],
        }
    }
}

/// Trait that should be defined for things that can be loaded into a sprite
pub trait TextureData {
    /// Returns sprite's data as series of RGBA bytes
    fn data(&self) -> Vec<u8>;

    /// Returns sprite's size in pixels
    fn size(&self) -> Vector2<u32>;
}

#[cfg(test)]
mod tests {
    use crate::renderer::Color;

    use super::RenderCommands;

    #[test]
    fn test_set_clear_color() {
        let mut render_commands = RenderCommands::default();
        render_commands.set_clear_color(Color::RED);
        assert_eq!(Some(Color::RED), render_commands.clear_color)
    }
}
