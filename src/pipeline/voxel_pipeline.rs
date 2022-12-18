use anyhow::*;
use cgmath::Matrix4;

use super::Pipeline;
use crate::bind_group::{BindGroupBuilder, Storage, GPUWrite};
use crate::camera::{Camera, CameraUniform};
use crate::model::ModelUniform;

pub struct FacesStorage {
    storage: Storage
}

impl FacesStorage {
    fn upload_slice(&self, queue: &wgpu::Queue, arr: &[u32]) {
        self.storage.update(queue, arr);
    }
}

impl From<Storage> for FacesStorage {
    fn from(storage: Storage) -> Self {
        Self {
            storage
        }
    }
}

pub struct VoxelPipeline {
    pipeline: Pipeline,
    camera_uniform: CameraUniform,
    model_uniform: ModelUniform,
    faces_storage: FacesStorage, 
}

impl VoxelPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        // Create the shader module
        let shader = device.create_shader_module(
            wgpu::include_wgsl!("../voxel.wgsl")
        );

        // Create the uniform group and the uniforms
        let mut builder = BindGroupBuilder::new(&device);
        let camera_uniform = CameraUniform::from(
            builder.create_uniform::<Matrix4<f32>>(wgpu::ShaderStages::VERTEX)
        );
        let model_uniform = ModelUniform::from(
            builder.create_uniform::<Matrix4<f32>>(wgpu::ShaderStages::VERTEX)
        );
        let faces_storage = FacesStorage::from(
            builder.create_storage::<[u32; 16 * 16 * 16]>(wgpu::ShaderStages::FRAGMENT)
        );
        let uniform_group = builder.build();

        Ok(Self {
            pipeline: Pipeline::new(device, format, uniform_group, shader)?,
            camera_uniform,
            model_uniform,
            faces_storage
        })
    }

    pub fn set_current<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.pipeline.set_current(render_pass);
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        camera: &Camera,
        position: cgmath::Vector3<f32>,
        faces_slice: &[u32]
    ) {
        self.camera_uniform.update_view_proj(queue, camera);
        self.model_uniform.update(queue, position);
        self.faces_storage.upload_slice(queue, faces_slice);
    }
}