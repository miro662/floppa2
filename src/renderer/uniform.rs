use cgmath::{Matrix4, Vector2};
use wgpu::{util::DeviceExt, BindGroupLayout};

use super::device::Gpu;
use lazy_static::lazy_static;

lazy_static! {
    static ref BOTTOM_LEFT_ZERO_MATRIX: Matrix4<f32> =
        Matrix4::from_scale(2.0) * Matrix4::from_translation((-0.5, -0.5, 0.0).into());
}

pub(crate) struct UniformBuffer {
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl UniformBuffer {
    pub(crate) fn new(gpu: &Gpu, size: Vector2<u32>) -> UniformBuffer {
        let device = gpu.device();

        let uniform = Uniform::for_screen_size(size);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });
        UniformBuffer {
            uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub(crate) fn change_size(&self, gpu: &Gpu, size: Vector2<u32>) {
        let uniform = Uniform::for_screen_size(size);
        gpu.queue()
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    view_proj: [[f32; 4]; 4],
}

impl Uniform {
    fn for_screen_size(size: Vector2<u32>) -> Self {
        let screen_size_scale =
            Matrix4::from_nonuniform_scale(1.0 / size.x as f32, 1.0 / size.y as f32, 1.0);

        let matrix = *BOTTOM_LEFT_ZERO_MATRIX * screen_size_scale;
        Uniform {
            view_proj: matrix.into(),
        }
    }
}
