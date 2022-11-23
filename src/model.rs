use wgpu::util::DeviceExt;

use crate::mesh::Mesh;

pub struct Model<'a> {
    mesh: Mesh<'a>,
    render_info: RenderInfo,
}

impl<'a> Model<'a> {
    pub fn new(device: &wgpu::Device, mesh: Mesh<'a>) -> Self {
        Self {
            render_info: RenderInfo::new(device, &mesh),
            mesh,
        }
    }

    pub fn render(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.render_info
            .render(self.mesh.indices_count(), render_pass);
    }
}

pub struct RenderInfo {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl RenderInfo {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        Self {
            vertex_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: mesh.vertex_data(),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            index_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: mesh.index_data(),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
        }
    }

    pub fn render<'a>(
        &'a self,
        indices_count: u32,
        render_pass: &mut wgpu::RenderPass<'a>
    ) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16
        );
        render_pass.draw_indexed(0..indices_count, 0, 0..1)
    }
}

struct ModelUniform {
    model_transform: [[f32; 4]; 4],
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}