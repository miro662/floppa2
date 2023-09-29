use wgpu::{include_wgsl, util::DeviceExt};

use super::{device::Gpu, instances::InstanceBuffer, uniform::UniformBuffer, BlitCommand};

pub(crate) struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    instance_buffer: InstanceBuffer,
    blit_buffer: wgpu::Buffer,
}

impl Pipeline {
    pub fn new(gpu: &Gpu, uniform: &UniformBuffer) -> Pipeline {
        let device = gpu.device();

        let shader_desc = include_wgsl!("shader.wgsl");
        let shader = device.create_shader_module(shader_desc);

        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[uniform.bind_group_layout()],
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

        let instance_buffer = InstanceBuffer::new(gpu);

        Pipeline {
            pipeline,
            blit_buffer,
            instance_buffer,
        }
    }

    pub fn render<'a>(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
        blits: impl Iterator<Item = &'a BlitCommand>,
        uniform: &UniformBuffer,
    ) -> wgpu::CommandBuffer {
        let device = gpu.device();

        self.instance_buffer.write_blits(gpu, blits);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if let Some(c) = clear_color {
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
            rpass.set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
            rpass.set_bind_group(0, uniform.bind_group(), &[]);
            rpass.draw(0..4, 0..self.instance_buffer.len() as _);
        }
        encoder.finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

const BLIT_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
    },
];

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        }
    }
}
