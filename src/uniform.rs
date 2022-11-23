use wgpu::util::DeviceExt;
use cgmath::{Matrix4, SquareMatrix};

pub trait UniformDataType: Sized {
    fn initial_value() -> Self;

    fn create_uniform(
        binding: u32,
        visibility: wgpu::ShaderStages,
        device: &wgpu::Device
    ) -> Uniform<Self> {
        let data = Self::initial_value();
        let debug_name = Self::debug_name();

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(debug_name),
                contents: data.as_slice(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(debug_name),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(debug_name),
        });
        
        Uniform { 
            data, 
            buffer, 
            bind_group,
            bind_group_layout
        }
    }

    fn as_slice<'a>(&self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of_val(self)
            )
        }
    }

    fn debug_name() -> &'static str {
        "Default uniform name"
    }
}

pub struct Uniform<DT: UniformDataType> {
    data: DT,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl<DT: UniformDataType> Uniform<DT> {
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update(&mut self, queue: &wgpu::Queue, data: DT) {
        queue.write_buffer(&self.buffer, 0, data.as_slice());
    }
}

impl UniformDataType for Matrix4<f32> {
    fn initial_value() -> Self {

        Matrix4::identity()
    }

    fn debug_name() -> &'static str {
        "Matrix uniform"
    }
}