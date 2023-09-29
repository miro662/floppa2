use wgpu::include_wgsl;

pub(crate) struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, target: wgpu::ColorTargetState) -> Pipeline {
        let shader_desc = include_wgsl!("shader.wgsl");
        let shader = device.create_shader_module(shader_desc);

        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);

        let targets = vec![Some(target)];
        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &targets,
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        };
        let pipeline = device.create_render_pipeline(&pipeline_desc);
        Pipeline { pipeline }
    }

    pub fn render(&self, device: &wgpu::Device, view: &wgpu::TextureView) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.draw(0..3, 0..1);
        }
        encoder.finish()
    }
}
