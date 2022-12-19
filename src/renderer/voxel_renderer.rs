use anyhow::*;
use cgmath::Vector3;

use crate::camera::Camera;
use crate::chunk::{VoxelMesh, Chunk};
use crate::pipeline::VoxelPipeline;
use crate::model::Model;

pub struct ChunkRenderer {
    model: Option<Model>,
    chunk_pipeline: VoxelPipeline,
    voxel_mesh: VoxelMesh,
}

impl ChunkRenderer {
    /// Create the model renderer, that renders a certain model
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat
    ) -> Result<Self> {
        Ok(Self {
            model: None,
            chunk_pipeline: VoxelPipeline::new(device, format)?,
            voxel_mesh: VoxelMesh::new()
        })
    }

    /// Change the model for a new one
    pub fn update_model<const L: usize, const H: usize>(
        &mut self,
        device: &wgpu::Device,
        chunk: &Chunk<L, H>
    ) {
        // Generate the full voxel mesh and store the new model
        self.voxel_mesh.serialize_chunk(&chunk);
        let mesh = self.voxel_mesh.mesh();
        self.model = Some(Model::new(device, mesh));
    }

    pub fn update_uniforms<
        const L: usize,
        const H: usize
    >(
        &mut self,
        queue: &wgpu::Queue,
        camera: &Camera,
        chunk: &Chunk<L, H>
    ) {
        self.chunk_pipeline.update(
            queue, camera,
            chunk.translation(),
            self.voxel_mesh.faces()
        );
    }

    /// Render all the instanced models on a single pass
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>
    ) {
        if let Some(model) = self.model.as_ref() {
            self.chunk_pipeline.set_current(render_pass);
            model.render(render_pass);
        }
    }
}