use anyhow::*;
use cgmath::Vector3;

use crate::camera::Camera;
use crate::chunk::{VoxelMesh, Chunk};
use crate::pipeline::ModelPipeline;
use crate::model::{ModelUniform, Model};

pub struct ChunkRenderer {
    model: Option<Model>,
    chunk_pipeline: ModelPipeline,
}

impl ChunkRenderer {
    /// Create the model renderer, that renders a certain model
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat
    ) -> Result<Self> {
        Ok(Self {
            model: None,
            chunk_pipeline: ModelPipeline::new(device, format)?
        })
    }

    /// Change the model for a new one
    pub fn update_model<const L: usize, const S: usize>(
        &mut self,
        device: &wgpu::Device,
        chunk: &Chunk<L, S>
    ) {
        let mut voxel_mesh = VoxelMesh::new();
        voxel_mesh.serialize_chunk(&chunk);
        let mesh = voxel_mesh.mesh();

        self.model = Some(Model::new(device, mesh));
    }

    pub fn update_uniforms(
        &mut self,
        queue: &wgpu::Queue,
        camera: &Camera
    ) {
        self.chunk_pipeline.update_uniforms(queue, camera, Vector3::new(0.0, 0.0, 0.0));
    }

    /// Render all the instanced models on a single pass
    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>
    ) {
        if let Some(model) = self.model.as_ref() {
            self.chunk_pipeline.set_current(render_pass);
            model.render(render_pass);
        }
    }
}