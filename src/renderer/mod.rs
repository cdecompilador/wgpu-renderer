use std::collections::HashMap;

use anyhow::*;

use crate::pipeline::ModelPipeline;
use crate::model::Model;
use crate::mesh::{Mesh, MeshBuilder};
use crate::chunk::{BlockPos, ChunkPos,Chunk, Block};
use crate::camera::Camera;
use crate::world::World;

mod model_renderer;
mod voxel_renderer;

pub use model_renderer::ModelRenderer;
pub use voxel_renderer::ChunkRenderer;

pub struct ChunksRenderer {
    renderers: HashMap<ChunkPos, ChunkRenderer>,
}

impl ChunksRenderer {
    pub fn new() -> Self {
        Self {
            renderers: HashMap::new(),
        }
    }

    pub fn load_chunk(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        chunk_pos: ChunkPos,
    ) -> Result<()> {
        self.renderers.insert(chunk_pos, ChunkRenderer::new(device, format)?);

        Ok(())
    }

    pub fn unload_chunk(&mut self, chunk_pos: ChunkPos) {
        self.renderers.remove(&chunk_pos);
    }
    
    pub fn update_chunk<
        const L: usize,
        const H: usize
    >(
        &mut self,
        device: &wgpu::Device,
        chunk: &Chunk<L, H>
    ) {
        let renderer = self.renderers.get_mut(&chunk.pos())
            .expect("ChunkRenderer not found");
        renderer.update_model(device, chunk);
    }

    pub fn prepare_chunk<
        const L: usize,
        const H: usize
    >(
        &mut self,
        queue: &wgpu::Queue,
        camera: &Camera,
        chunk: &Chunk<L, H>
    ) {
       let renderer = self.renderers.get_mut(&chunk.pos())
            .expect("ChunkRenderer not found");
        renderer.update_uniforms(queue, camera, chunk);
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>
    ) {
        for renderer in self.renderers.values() {
            renderer.render(render_pass);
        }
    }
}

pub struct MasterRenderer {
    format: wgpu::TextureFormat,

    /// Color used to clear the screen
    clear_color: wgpu::Color,

    /// The chunk data
    /// TODO: In the future this should be outside the renderer
    chunk: Chunk<16, 16>,
    chunk2: Chunk<16, 16>,

    /// Renderer that can render a chunk
    // chunk_renderer: ChunkRenderer,
    // chunk_renderer2: ChunkRenderer,
    chunks_renderer: ChunksRenderer,

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
            format,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.4,
                a: 1.0
            },
            chunk: {
                let mut chunk = Chunk::new(ChunkPos::new(0, -1));
                for x in 0..16 {
                    for y in x..16 {
                        for z in 0..16 {
                            chunk.place_block(BlockPos::new(x, y, z), Block::Dirt);
                        }
                    }
                }
                chunk
            },
            chunk2: {
                let mut chunk = Chunk::new(ChunkPos::new(1, 0));
                for x in 0..16 {
                    for y in x..16 {
                        for z in 0..16 {
                            chunk.place_block(BlockPos::new(x, y, z), Block::Dirt);
                        }
                    }
                }
                chunk
            },
            // chunk_renderer: ChunkRenderer::new(device, format)?,
            // chunk_renderer2: ChunkRenderer::new(device, format)?,
            chunks_renderer: ChunksRenderer::new(),
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
        world: &mut World
    ) {
        for chunk in world.scheduled_chunks() {
            self.chunks_renderer.load_chunk(device, self.format, chunk.pos()).unwrap();
            self.chunks_renderer.update_chunk(device, chunk);
        }
        for chunk in world.to_update_chunks() {
            self.chunks_renderer.update_chunk(device, chunk);
        }
        // self.chunk_renderer.update_model(device, &self.chunk);
        // self.chunk_renderer2.update_model(device, &self.chunk2);
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
        self.chunks_renderer.render(&mut render_pass);
        // self.chunk_renderer2.render(&mut render_pass);
        self.m1_pipeline.set_current(&mut render_pass);
        self.m1.render(&mut render_pass);
    }

    pub fn clear_color(&self) -> wgpu::Color {
        self.clear_color
    }
    
    /// Update all the uniforms with refiened input in order
    pub fn update_uniforms<
        'a,
        const L: usize,
        const H: usize
    >(
        &'a mut self,
        queue: &wgpu::Queue,
        camera: &Camera,
        chunks: impl Iterator<Item = &'a Chunk<L, H>>
    ) {
        for chunk in chunks {
            self.chunks_renderer.prepare_chunk(queue, camera, chunk);
        }
        // self.chunk_renderer.update_uniforms(queue, camera, &self.chunk);
        // self.chunk_renderer2.update_uniforms(queue, camera, &self.chunk2);
        self.m1_pipeline.update_uniforms(queue, camera, cgmath::Vector3::new(0.0, 0.0, 0.0));
    }
}