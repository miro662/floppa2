mod device;
mod instances;
mod pipeline;
mod uniform;

use std::{
    sync::mpsc::{self, Receiver},
    thread, vec,
};

use cgmath::Vector2;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use self::{device::Gpu, pipeline::Pipeline, uniform::UniformBuffer};

pub type Color = wgpu::Color;

#[derive(Debug, Clone)]
enum RenderThreadMessage {
    Resize(Vector2<u32>),
    Render(RenderCommands),
}

struct RendererThread {
    gpu: Gpu,
    pipeline: Pipeline,
    uniform: UniformBuffer,
}

impl RendererThread {
    fn compatible_with<T>(window: T, size: impl Into<Vector2<u32>>) -> RendererThread
    where
        T: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let size = size.into();
        let gpu = Gpu::compatible_with(window, size);
        let uniform = UniformBuffer::new(&gpu, size);
        let pipeline = Pipeline::new(&gpu, &uniform);

        RendererThread {
            gpu,
            pipeline,
            uniform,
        }
    }

    fn run(mut self, rx: Receiver<RenderThreadMessage>) {
        for command in rx {
            match command {
                RenderThreadMessage::Resize(size) => self.resize(size),
                RenderThreadMessage::Render(command) => self.render(command),
            }
        }
    }

    fn resize(&mut self, size: Vector2<u32>) {
        self.gpu.resize(size);
        self.uniform.change_size(&self.gpu, size);
    }

    fn render(&mut self, command: RenderCommands) {
        let frame = self.gpu.surface().get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.gpu.queue().submit(Some(self.pipeline.render(
            &self.gpu,
            &view,
            command.clear_color,
            command.blits.iter(),
            &self.uniform,
        )));
        frame.present();
    }
}

/// Allows communicating with renderer thread
pub struct Renderer {
    tx: mpsc::Sender<RenderThreadMessage>,
}

impl Renderer {
    /// Creates renderer compatible with provided window and size
    pub fn compatible_with<T>(window: T, size: (u32, u32)) -> Renderer
    where
        T: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let renderer_thread = RendererThread::compatible_with(window, size);
        let (tx, rx) = mpsc::channel();
        thread::spawn(|| renderer_thread.run(rx));
        Renderer { tx }
    }

    /// Notifies renderer about window's resize
    pub fn resize(&self, size: Vector2<u32>) {
        self.tx.send(RenderThreadMessage::Resize(size)).unwrap();
    }

    /// Render things described by callback to a window
    pub fn render(&self, callback: impl FnOnce(&mut RenderCommands)) {
        let mut target = RenderCommands {
            clear_color: Some(Color::BLACK),
            blits: vec![],
        };
        callback(&mut target);
        self.tx.send(RenderThreadMessage::Render(target)).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct BlitCommand {
    pub(crate) position: Vector2<u32>,
    pub(crate) size: Vector2<u32>,
    pub(crate) color: Color,
}

impl BlitCommand {
    pub fn at(&mut self, position: impl Into<Vector2<u32>>) -> &mut Self {
        self.position = position.into();
        self
    }

    pub fn with_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}

/// Allows operations on target
#[derive(Debug, Clone)]
pub struct RenderCommands {
    clear_color: Option<Color>,
    blits: Vec<BlitCommand>,
}

impl RenderCommands {
    /// Changes color that is used to clear target before rendering
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = Some(color);
    }

    pub fn draw_rectangle(&mut self, size: impl Into<Vector2<u32>>) -> &mut BlitCommand {
        let blit_command = BlitCommand {
            position: (0, 0).into(),
            size: size.into(),
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
