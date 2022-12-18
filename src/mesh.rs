use std::mem;
use std::borrow::Cow;

use crate::chunk::BlockPos;

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    vertices: Cow<'static, [Vertex]>,
    indices: Cow<'static, [u16]>,
}

macro_rules! face {
    ($name:ident, $v1:expr, $v2:expr, $v3:expr, $v4:expr) => {
        pub const $name: Self = Mesh {
            vertices: Cow::Borrowed(&[
                Vertex::new($v1, [1.0, 0.0, 0.0]),
                Vertex::new($v2, [0.0, 1.0, 0.0]),
                Vertex::new($v3, [0.0, 0.0, 1.0]),
                Vertex::new($v4, [1.0, 1.0, 1.0]),
            ]),
            indices: Cow::Borrowed(&[0, 1, 2, 0, 3, 1])
        };
    }
}

impl Mesh {
    pub const QUAD: Self = Mesh {
        vertices: Cow::Borrowed(&[
            Vertex::new([-0.5, 0.5, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([-0.5, -0.5, 0.0], [0.0, 1.0, 0.0]),
            Vertex::new([0.5, -0.5, 0.0], [0.0, 0.0, 1.0]),
            Vertex::new([0.5, 0.5, 0.0], [1.0, 1.0, 1.0]),
        ]),
        indices: Cow::Borrowed(&[0, 1, 2, 0, 2, 3]),
    };

    face!(UP_FACE, 
        [-0.5,  0.5,  0.5],
        [ 0.5,  0.5, -0.5],
        [-0.5,  0.5, -0.5],
        [ 0.5,  0.5,  0.5]);

    face!(DOWN_FACE, 
        [-0.5, -0.5,  0.5],
        [ 0.5, -0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [ 0.5, -0.5,  0.5]);

    face!(LEFT_FACE, 
        [-0.5, -0.5,  0.5],
        [-0.5,  0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [-0.5,  0.5,  0.5]);
        
    face!(RIGHT_FACE, 
        [ 0.5, -0.5,  0.5],
        [ 0.5,  0.5, -0.5],
        [ 0.5, -0.5, -0.5],
        [ 0.5,  0.5,  0.5]);

    face!(FRONT_FACE, 
        [ 0.5, -0.5, -0.5],
        [-0.5,  0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [ 0.5,  0.5, -0.5]);
        
    face!(BACK_FACE, 
        [ 0.5, -0.5,  0.5],
        [-0.5,  0.5,  0.5],
        [-0.5, -0.5,  0.5],
        [ 0.5,  0.5,  0.5]);

    #[allow(dead_code)]
    pub const TRIANGLE: Self = Mesh {
        vertices: Cow::Borrowed(&[
            Vertex::new([0.5, -0.5, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([0.0, 0.5, 0.0], [0.0, 1.0, 0.0]),
            Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0, 1.0]),
        ]),
        indices: Cow::Borrowed(&[0, 1, 2]),
    };

    #[allow(dead_code)]
    pub const PENTAGON: Self = Mesh {
        vertices: Cow::Borrowed(&[
            Vertex::new([-0.0868241, 0.49240386, 0.0], [0.0, 0.0, 0.0]), 
            Vertex::new([-0.49513406, 0.06958647, 0.0], [0.0, 0.0, 0.0]),
            Vertex::new([-0.21918549, -0.44939706, 0.0], [0.0, 0.0, 0.0]), 
            Vertex::new([0.35966998, -0.3473291, 0.0], [0.0, 0.0, 0.0]),
            Vertex::new([0.44147372, 0.2347359, 0.0], [0.0, 0.0, 0.0]),
        ]),
        indices:  Cow::Borrowed(
                    &[0, 1, 4,
                      1, 2, 4,
                      2, 3, 4])
    };

    pub const WEIRD: Self = Mesh {
        vertices: Cow::Borrowed(&[
            Vertex::new([-0.5, -0.5, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([0.0,  -0.5, 0.0], [0.0, 1.0, 0.0]),
            Vertex::new([-0.5, 0.0, 0.0], [0.0, 0.0, 1.0]),
            Vertex::new([0.5, 0.5, 0.0], [1.0, 0.0, 0.0]),
            Vertex::new([0.0,  0.5, 0.0], [0.0, 1.0, 0.0]),
            Vertex::new([0.5, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ]),
        indices: Cow::Borrowed(&[0, 1, 2, 3, 4, 5])
    };

    pub fn new(
        vertices: impl Into<Cow<'static, [Vertex]>>,
        indices: impl Into<Cow<'static, [u16]>>
    ) -> Self {
        Self {
            vertices: vertices.into(),
            indices: indices.into()
        }
    }

    pub fn indices_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn vertex_data<'a>(&'a self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.vertices.as_ref().as_ptr() as *const u8,
                std::mem::size_of_val(self.vertices.as_ref()),
            )
        }
    }

    pub fn index_data<'a>(&'a self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.indices.as_ref().as_ptr() as *const u8,
                std::mem::size_of_val(self.indices.as_ref()),
            )
        }
    }
}

pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    curr_idx: u16
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self { 
            vertices: Vec::new(),
            indices: Vec::new(),
            curr_idx: 0
        }
    }

    pub fn push(&mut self, mut mesh: Mesh, position: BlockPos) {
        let mut max_idx = 0;
        for index in mesh.indices.iter() {
            max_idx = u16::max(max_idx, *index);
            self.indices.push(self.curr_idx + *index);
        }
        self.curr_idx += max_idx + 1;
        for vertex in mesh.vertices.iter() {
            self.vertices.push(vertex.translate(position));
        }
    }

    pub fn build(self) -> Mesh {
        Mesh::new(self.vertices, self.indices)
    }
}

pub const VERTEX_DESC: wgpu::VertexBufferLayout<'static> = 
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            }
        ]
    };

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            color
        }
    }

    const fn from_blockpos(BlockPos { x, y, z }: BlockPos, color: [f32; 3]) -> Self {
        Self { 
            position: [x as f32, y as f32, z as f32],
            color,
        }
    }

    pub fn translate(&self, position: BlockPos) -> Self {
        let [mut x, mut y, mut z] = self.position;
        x += position.x as f32;
        y += position.y as f32;
        z += position.z as f32;

        Self {
            position: [x, y, z],
            color: self.color
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut m = MeshBuilder::new();
        m.push(Mesh::UP_FACE, BlockPos::new(0, 0, 0));
        m.push(Mesh::DOWN_FACE, BlockPos::new(0, 0, 0));
        m.push(Mesh::RIGHT_FACE, BlockPos::new(0, 0, 0));
        let m = m.build();

        pretty_assertions::assert_eq!(
            m,
            Mesh::new(
                vec![
                    Vertex::new([-0.5,  0.5,  0.5], [1.0, 0.0, 0.0]),
                    Vertex::new([ 0.5,  0.5, -0.5], [0.0, 1.0, 0.0]),
                    Vertex::new([-0.5,  0.5, -0.5], [0.0, 0.0, 1.0]),
                    Vertex::new([ 0.5,  0.5,  0.5], [1.0, 1.0, 1.0]),

                    Vertex::new([-0.5, -0.5,  0.5], [1.0, 0.0, 0.0]),
                    Vertex::new([ 0.5, -0.5, -0.5], [0.0, 1.0, 0.0]),
                    Vertex::new([-0.5, -0.5, -0.5], [0.0, 0.0, 1.0]),
                    Vertex::new([ 0.5, -0.5,  0.5], [1.0, 1.0, 1.0]),

                    Vertex::new([ 0.5, -0.5,  0.5], [1.0, 0.0, 0.0]),
                    Vertex::new([-0.5,  0.5,  0.5], [0.0, 1.0, 0.0]),
                    Vertex::new([-0.5, -0.5,  0.5], [0.0, 0.0, 1.0]),
                    Vertex::new([ 0.5,  0.5,  0.5], [1.0, 1.0, 1.0]),
                ],
                vec![
                    0, 1, 2, 0, 3, 1,
                    4, 5, 6, 4, 7, 5,
                    8, 9, 10, 8, 11, 9
                ]
            )
        );
        assert_eq!(m.indices_count(), 18);
    }

}
