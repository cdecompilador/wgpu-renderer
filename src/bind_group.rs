use std::rc::Rc;

use wgpu::util::DeviceExt;
use cgmath::{Matrix4, SquareMatrix};

pub trait GPUDataType: Sized {
    fn initial_value() -> Self;

    fn create_uniform(
        device: &wgpu::Device
    ) -> Uniform {
        let data = Self::initial_value();
        let debug_name = Self::debug_name();

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(debug_name),
                contents: data.as_slice(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        Uniform { 
            buffer: Rc::new(buffer), 
        }
    }

    fn create_storage(
        device: &wgpu::Device
    ) -> Storage {
        let data = Self::initial_value();
        let debug_name = Self::debug_name();

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(debug_name),
                contents: data.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
            }
        );

        Storage {
            buffer: Rc::new(buffer)
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

impl GPUDataType for Matrix4<f32> {
    fn initial_value() -> Self {
        Matrix4::identity()
    }

    fn debug_name() -> &'static str {
        "GPU Matrix4"
    }
}

impl<const N: usize> GPUDataType for [Matrix4<f32>; N] {
    fn initial_value() -> Self {
        [Matrix4::identity(); N]
    }

    fn debug_name() -> &'static str {
        "GPU Matrix4 array"
    }

    fn as_slice<'a>(&self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * std::mem::size_of::<Matrix4<f32>>()
            )
        }
    }
}

impl<const N: usize> GPUDataType for [u32; N] {
    fn initial_value() -> Self {
        [0; N]
    }

    fn debug_name() -> &'static str {
        "GPU u32 3D array"
    }
}

impl GPUDataType for &[u32] {
    fn initial_value() -> Self {
        unreachable!()
    }

    fn debug_name() -> &'static str {
        unreachable!()
    }
    
    fn as_slice<'a>(&self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * std::mem::size_of::<u32>()
            )
        }
    }
}

pub struct Storage {
    buffer: Rc<wgpu::Buffer>
}

impl Storage {
    pub fn buffer(&self) -> Rc<wgpu::Buffer> {
        self.buffer.clone()
    }
}

pub struct Uniform {
    buffer: Rc<wgpu::Buffer>,
}

impl Uniform {
    pub fn buffer(&self) -> Rc<wgpu::Buffer> {
        self.buffer.clone()
    }
}

pub trait GPUWrite<DT: GPUDataType> {
    fn update(&self, queue: &wgpu::Queue, data: DT) {
        queue.write_buffer(self.buffer(), 0, data.as_slice())
    }

    fn buffer(&self) -> &wgpu::Buffer;
}

macro_rules! impl_gpuwrite {
    ($t:ty) => {
        impl<DT: GPUDataType> GPUWrite<DT> for $t {
            fn buffer(&self) -> &wgpu::Buffer {
                &self.buffer
            }
        }
    }
}

impl_gpuwrite!(Storage);
impl_gpuwrite!(Uniform);

pub struct BindGroup {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout
}

impl BindGroup {
    pub fn bind_group_layout<'a>(&'a self) -> &'a wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
    
    pub fn bind_group<'a>(&'a self) -> &'a wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct BindGroupBuilder<'a> {
    device: &'a wgpu::Device,
    bind_count: u32,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    entries: Vec<wgpu::BindGroupEntry<'static>>
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            bind_count: 0,
            layout_entries: Vec::new(),
            entries: Vec::new()
        }
    }

    pub fn register_texture(
        &mut self,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler
    ) {
        // Create the bindings
        let texture_binding = self.get_binding();
        let sampler_binding = self.get_binding();

        // Generate the information to later instantiate the full bind group
        self.layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: texture_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float {
                        filterable: true
                    }
                },
                count: None,
            }
        );
        self.layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: sampler_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(
                    wgpu::SamplerBindingType::Filtering
                ),
                count: None,
            }
        );
        self.entries.push(
            wgpu::BindGroupEntry {
                binding: texture_binding,
                resource: unsafe {
                    std::mem::transmute::<
                        wgpu::BindingResource<'_>,
                        wgpu::BindingResource<'static>
                    >(
                        wgpu::BindingResource::TextureView(view)
                    )
                }
            }
        );
        self.entries.push(
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: unsafe {
                    std::mem::transmute::<
                        wgpu::BindingResource<'_>,
                        wgpu::BindingResource<'static>
                    >(
                        wgpu::BindingResource::Sampler(sampler)
                    )
                }
            }
        );
    }

    pub fn create_uniform<DT>(
        &mut self,
        visibility: wgpu::ShaderStages
    ) -> Uniform
    where
        DT: GPUDataType + 'static
    { 
        // Get its associated binding id
        let binding = self.get_binding();

        // Instantiate the uniform and save it
        let uniform = DT::create_uniform(self.device);
        let buffer = uniform.buffer();
        
        // Generate the information to later instantiate the full bind group
        self.layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }
        );
        self.entries.push(
            wgpu::BindGroupEntry {
                binding,
                resource: unsafe { 
                    std::mem::transmute::<
                        wgpu::BindingResource<'_>,
                        wgpu::BindingResource<'static>
                    >(
                        buffer.as_entire_binding()
                    )
                },
            }
        );

        uniform
    }
    
    pub fn create_storage<DT>(
        &mut self,
        visibility: wgpu::ShaderStages
    ) -> Storage
    where
        DT: GPUDataType + 'static
    { 
        // Get its associated binding id
        let binding = self.get_binding();

        // Instantiate the uniform and save it
        let storage = DT::create_storage(self.device);
        let buffer = storage.buffer();
        
        // Generate the information to later instantiate the full bind group
        self.layout_entries.push(
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { 
                        read_only: true 
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }
        );
        self.entries.push(
            wgpu::BindGroupEntry {
                binding,
                resource: unsafe { 
                    std::mem::transmute::<
                        wgpu::BindingResource<'_>,
                        wgpu::BindingResource<'static>
                    >(
                        buffer.as_entire_binding()
                    )
                },
            }
        );

        storage
    }

    pub fn build(self) -> BindGroup {
        let bind_group_layout = self.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: self.layout_entries.as_slice(),
                label: None,
            }
        );
        let bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: self.entries.as_slice(),
                label: None,
            }
        );

        BindGroup {
            bind_group_layout,
            bind_group
        }
    }

    fn get_binding(&mut self) -> u32 {
        let res = self.bind_count;
        self.bind_count += 1;
        res
    }
}

