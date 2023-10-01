use wgpu::{include_wgsl, util::DeviceExt, CommandBuffer, VertexAttribute};

use crate::renderer::texture_ref::TextureRef;

use super::{
    buffers::instances::InstanceBuffer, buffers::uniform::UniformBuffer, gpu::Gpu,
    textures::Textures,
};

pub(crate) struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    blit_buffer: wgpu::Buffer,
}

pub(crate) struct PipelineBuffers<'a> {
    pub(crate) uniform: &'a UniformBuffer,
    pub(crate) textures: &'a Textures,
    pub(crate) instances: &'a InstanceBuffer,
}

pub(crate) struct RenderPass<'a> {
    pub(crate) buffers: PipelineBuffers<'a>,
    pub(crate) view: &'a wgpu::TextureView,
    pub(crate) clear_color: Option<wgpu::Color>,
    pub(crate) texture: TextureRef,
    pub(crate) instances: std::ops::Range<u32>,
}

impl Pipeline {
    pub fn new(gpu: &Gpu, buffers: PipelineBuffers<'_>) -> Pipeline {
        let device = gpu.device();

        let shader_desc = include_wgsl!("shader.wgsl");
        let shader = device.create_shader_module(shader_desc);

        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                buffers.uniform.bind_group_layout(),
                buffers.textures.bind_group_layout(),
            ],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);

        let targets = vec![Some(gpu.surface_format().into())];
        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), InstanceBuffer::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        };
        let pipeline = device.create_render_pipeline(&pipeline_desc);

        let blit_buffer_desc = wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(BLIT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let blit_buffer = device.create_buffer_init(&blit_buffer_desc);

        Pipeline {
            pipeline,
            blit_buffer,
        }
    }

    pub fn encode_pass(&mut self, gpu: &Gpu, pass: RenderPass<'_>) -> CommandBuffer {
        let device = gpu.device();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: pass.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if let Some(c) = pass.clear_color {
                            wgpu::LoadOp::Clear(c)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.blit_buffer.slice(..));
            rpass.set_vertex_buffer(1, pass.buffers.instances.buffer().slice(..));
            rpass.set_bind_group(0, pass.buffers.uniform.bind_group(), &[]);
            rpass.set_bind_group(
                1,
                pass.buffers.textures.bind_group_for_texture(&pass.texture),
                &[],
            );
            rpass.draw(0..4, pass.instances as _);
        }
        encoder.finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uv_position: [f32; 2],
}

const BLIT_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        uv_position: [0.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        uv_position: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        uv_position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        uv_position: [1.0, 0.0],
    },
];

impl Vertex {
    const ATTR_ARRAY: [VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTR_ARRAY,
        }
    }
}
