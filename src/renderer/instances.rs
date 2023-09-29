use cgmath::Matrix4;
use wgpu::{BufferUsages, VertexAttribute};

use super::{device::Gpu, BlitCommand};
use std::mem;

const INSTANCE_BUFFER_CAPACITY: u64 = 1 << 12;

pub(crate) struct InstanceBuffer {
    buffer: wgpu::Buffer,
    len: u64,
}

impl InstanceBuffer {
    pub(crate) fn new(gpu: &Gpu) -> InstanceBuffer {
        let buffer_descriptor = wgpu::BufferDescriptor {
            label: None,
            size: INSTANCE_BUFFER_CAPACITY * (mem::size_of::<Instance>() as u64),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        };
        let buffer = gpu.device().create_buffer(&buffer_descriptor);
        InstanceBuffer { buffer, len: 0 }
    }

    pub(crate) fn write_blits<'a>(
        &mut self,
        gpu: &Gpu,
        instances: impl Iterator<Item = &'a BlitCommand>,
    ) {
        let instance_data: Vec<_> = instances.map(Instance::from_blit).collect();
        gpu.queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&instance_data));
        self.len = instance_data.len() as u64;
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn len(&self) -> u64 {
        self.len
    }

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Instance::ATTR_ARRAY,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Instance {
    const ATTR_ARRAY: [VertexAttribute; 5] = wgpu::vertex_attr_array![10 => Float32x4, 11 => Float32x4, 12 => Float32x4, 13 => Float32x4, 14 => Float32x4];

    fn from_blit(blit: &BlitCommand) -> Instance {
        let translation_matrix =
            Matrix4::from_translation((blit.position.x as f32, blit.position.y as f32, 0.0).into());
        let scale_matrix =
            Matrix4::from_nonuniform_scale(blit.size.x as f32, blit.size.y as f32, 1.0);
        let instance_matrix = translation_matrix * scale_matrix;
        Instance {
            model: instance_matrix.into(),
            color: [
                blit.color.r as f32,
                blit.color.g as f32,
                blit.color.b as f32,
                blit.color.a as f32,
            ],
        }
    }
}
