use anyhow::*;
use cgmath::Vector3;

use crate::pipeline::ModelPipeline;
use crate::model::Model;
use crate::mesh::{Mesh, MeshBuilder};
use crate::chunk::{BlockPos, Chunk, Block};
use crate::camera::Camera;

mod model_renderer;
mod voxel_renderer;

pub use model_renderer::ModelRenderer;
pub use voxel_renderer::ChunkRenderer;

pub struct MasterRenderer {
    /// Color used to clear the screen
    clear_color: wgpu::Color,

    /// The chunk data
    /// TODO: In the future this should be outside the renderer
    chunk: Chunk<16, 16>,

    /// Renderer that can render a chunk
    chunk_renderer: ChunkRenderer,

    // Test figure just to mark the center of the world
    m1: Model,
    m1_pipeline: ModelPipeline,
}

impl MasterRenderer {
    /// Create a `MasterRenderer` for a certain SurfaceTexture
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        Ok(Self {
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.4,
                a: 1.0
            },
            chunk: {
                let mut chunk = Chunk::new();
                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            chunk.place_block(BlockPos::new(x, y, z), Block::Dirt);
                        }
                    }
                }
                chunk
            },
            chunk_renderer: ChunkRenderer::new(device, format)?,
            m1_pipeline: ModelPipeline::new(
                device,
                format,
            )?,
            m1: Model::new(device, {
                let mut builder = MeshBuilder::new();
                builder.push(Mesh::WEIRD, BlockPos::new(0, 0, 0));
                // builder.push(Mesh::BACK_FACE, BlockPos::new(0, 0, 0));
                // builder.push(Mesh::RIGHT_FACE, BlockPos::new(0, 0, 0));
                // builder.push(Mesh::LEFT_FACE, BlockPos::new(0, 0, 0));
                // builder.push(Mesh::UP_FACE, BlockPos::new(0, 0, 0));
                // builder.push(Mesh::DOWN_FACE, BlockPos::new(0, 0, 0));
                // builder.push(Mesh::FRONT_FACE, BlockPos::new(1, 0, 0));
                // builder.push(Mesh::BACK_FACE, BlockPos::new(1, 0, 0));
                // builder.push(Mesh::RIGHT_FACE, BlockPos::new(1, 0, 0));
                // builder.push(Mesh::LEFT_FACE, BlockPos::new(1, 0, 0));
                // builder.push(Mesh::UP_FACE, BlockPos::new(1, 0, 0));
                // builder.push(Mesh::DOWN_FACE, BlockPos::new(1, 0, 0));
                // builder.push(Mesh::FRONT_FACE, BlockPos::new(1, 1, 0));
                // builder.push(Mesh::BACK_FACE, BlockPos::new(1, 1, 0));
                // builder.push(Mesh::RIGHT_FACE, BlockPos::new(1, 1, 0));
                // builder.push(Mesh::LEFT_FACE, BlockPos::new(1, 1, 0));
                // builder.push(Mesh::UP_FACE, BlockPos::new(1, 1, 0));
                // builder.push(Mesh::DOWN_FACE, BlockPos::new(1, 1, 0));
                builder.build()
            }),
        })
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
    ) {
        self.chunk_renderer.update_model(device, &self.chunk);
    }

    /// Main rendering, creates the render pass and manages the order of 
    /// pipelines/models and subrenderers
    pub fn render<'a>(
        &'a mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView
    ) {
        // Clear
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(
                        self.clear_color()
                    ),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                }
            ),
        });

        // Draw
        self.chunk_renderer.render(&mut render_pass);
        self.m1_pipeline.set_current(&mut render_pass);
        self.m1.render(&mut render_pass);
    }

    pub fn clear_color(&self) -> wgpu::Color {
        self.clear_color
    }
    
    /// Update all the uniforms with refiened input in order
    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, camera: &Camera) {
        self.chunk_renderer.update_uniforms(queue, camera);
        self.m1_pipeline.update_uniforms(queue, camera, cgmath::Vector3::new(0.0, 0.0, 0.0));
    }
}