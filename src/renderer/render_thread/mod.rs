mod buffers;
mod gpu;
mod pipeline;
mod textures;

use std::{collections::HashMap, sync::mpsc::Receiver};

use crate::renderer::render_thread::buffers::instances::Instance;

use self::{
    buffers::instances::InstanceBuffer,
    buffers::uniform::UniformBuffer,
    gpu::Gpu,
    pipeline::{Pipeline, PipelineBuffers, RenderPass},
    textures::Textures,
};
use cgmath::Vector2;

use super::{texture_ref::TextureRef, CompatibleWindow, RenderCommands};

#[derive(Debug)]
pub(crate) enum RenderThreadMessage {
    Resize(Vector2<u32>),
    Render(RenderCommands),
    LoadTexture(TextureRef, Vec<u8>, Vector2<u32>),
}

pub(crate) struct RendererThread {
    gpu: Gpu,
    pipeline: Pipeline,
    uniform: UniformBuffer,
    textures: Textures,
    instances: InstanceBuffer,
}

impl RendererThread {
    pub(crate) fn compatible_with(
        window: impl CompatibleWindow,
        size: impl Into<Vector2<u32>>,
    ) -> RendererThread {
        let size = size.into();
        let gpu = Gpu::compatible_with(window, size);
        let uniform = UniformBuffer::new(&gpu, size);
        let texutres = Textures::new(&gpu);
        let instances = InstanceBuffer::new(&gpu);
        let pipeline = Pipeline::new(
            &gpu,
            PipelineBuffers {
                uniform: &uniform,
                textures: &texutres,
                instances: &instances,
            },
        );

        RendererThread {
            gpu,
            pipeline,
            uniform,
            textures: texutres,
            instances,
        }
    }

    pub(crate) fn run(mut self, rx: Receiver<RenderThreadMessage>) {
        for command in rx {
            match command {
                RenderThreadMessage::Resize(size) => self.resize(size),
                RenderThreadMessage::Render(command) => self.render(command),
                RenderThreadMessage::LoadTexture(id, data, size) => {
                    self.textures.load_texture(&self.gpu, id, &data, size)
                }
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

        let mut render_passes = HashMap::new();
        for blit in command.blits {
            render_passes
                .entry(blit.texture_id)
                .or_insert_with(Vec::new)
                .push(Instance::from_blit(&blit));
        }

        let mut data = vec![];
        let mut render_ranges = HashMap::new();
        for (texture_id, instances) in render_passes.iter_mut() {
            let from = data.len() as u32;
            let to = from + instances.len() as u32;
            render_ranges.insert(texture_id, from..to);
            data.append(instances)
        }

        self.instances.write_instances(&self.gpu, &data);

        let command_buffers = render_ranges
            .iter()
            .enumerate()
            .map(|(i, (texture, range))| {
                let clear_color = if i == 0 { command.clear_color } else { None };

                let pass = RenderPass {
                    buffers: PipelineBuffers {
                        uniform: &self.uniform,
                        textures: &self.textures,
                        instances: &self.instances,
                    },
                    view: &view,
                    clear_color,
                    texture: **texture,
                    instances: range.clone(),
                };
                self.pipeline.encode_pass(&self.gpu, pass)
            });

        self.gpu.queue().submit(command_buffers);
        frame.present();
    }
}
