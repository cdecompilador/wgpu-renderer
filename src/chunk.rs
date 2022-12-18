use std::ops::Deref;
use std::fmt;

use crate::mesh::{Mesh, MeshBuilder};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Front = 0,
    Back  = 1,
    Up    = 2,
    Down  = 3,
    Left  = 4,
    Right = 5
}

impl Face {
    fn mesh(&self) -> Mesh {
        match *self {
            Face::Front => Mesh::FRONT_FACE,
            Face::Back  => Mesh::BACK_FACE,
            Face::Up    => Mesh::UP_FACE,
            Face::Down  => Mesh::DOWN_FACE,
            Face::Left  => Mesh::LEFT_FACE,
            Face::Right => Mesh::RIGHT_FACE
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: usize,
    pub y: usize,
    pub z: usize
}

impl BlockPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self {
            x,
            y,
            z
        }
    }
}

pub struct VoxelMesh {
    faces: Vec<Face>,
    positions: Vec<BlockPos>
}

macro_rules! add_face {
    ($face:expr, $block:expr, $faces:expr, $positions:expr) => {
        let pos = $block.block_pos;
        let neighbor = $block.neighbor($face);
        if let Some(neighbor) = neighbor {
            if *neighbor == Block::Air {
                $faces.push($face);
                $positions.push(pos);
            }
        } else {
            $faces.push($face);
            $positions.push(pos);
        }
    }
}

impl VoxelMesh {
    pub fn new() -> Self {
        Self {
            faces: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn serialize_chunk<
        const L: usize,
        const H: usize
    >(&mut self, chunk: &Chunk<L, H>) {
        self.faces.clear();
        self.positions.clear();

        for block in chunk.iter() {
            if *block == Block::Air {
                continue;
            }

            add_face!(Face::Front, block, self.faces, self.positions);
            add_face!(Face::Back, block, self.faces, self.positions);
            add_face!(Face::Up, block, self.faces, self.positions);
            add_face!(Face::Down, block, self.faces, self.positions);
            add_face!(Face::Left, block, self.faces, self.positions);
            add_face!(Face::Right, block, self.faces, self.positions);
        }
    }

    pub fn faces<'a>(&'a self) -> &'a [u32] {
        unsafe {
            std::slice::from_raw_parts(
                self.faces.as_slice().as_ptr() as *const u32,
                self.faces.len()
            )
        }
    }

    pub fn mesh(&mut self) -> Mesh {
        // Assertions to ensure proper optimizations
        assert_eq!(self.faces.len(), self.positions.len());

        // TODO: Remove the faces that are not visible

        // Convert those faces to a mesh
        let mut builder = MeshBuilder::new();
        for (face, position) in self.faces.iter().zip(self.positions.iter()) {
            builder.push(face.mesh(), *position);
        }

        builder.build()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Dirt,
    Id(u8)
}

impl Block {
    pub fn color(&self) -> [f32; 3] {
        match *self {
            Block::Air => unreachable!(),
            Block::Dirt => [0.5, 0.5, 0.5],
            Block::Id(_) => [1.0, 0.1, 0.1]
        }
    }
}

pub struct BlockRef<'a, const L: usize, const H: usize> {
    chunk_ref: &'a Chunk<L, H>,
    block_ref: &'a Block,
    block_pos: BlockPos
}

impl<'a, const L: usize, const H: usize> fmt::Debug for BlockRef<'a, L, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockRef")
            .field("block", self.block_ref)
            .finish()
    }
}

impl<'a, const L: usize, const H: usize> Deref for BlockRef<'a, L, H> {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        self.block_ref
    }
}

impl<'a, const L: usize, const H: usize> BlockRef<'a, L, H> {
    pub fn neighbor(&self, face: Face) -> Option<BlockRef<'a, L, H>> {
        let Self { block_pos: BlockPos { x, y, z }, .. } = *self;
        match face {
            Face::Front => {
                self.chunk_ref.index_block(BlockPos::new(x, y, z.checked_sub(1)?))
            }
            Face::Back => {
                self.chunk_ref.index_block(BlockPos::new(x, y, z + 1))
            }
            Face::Right => {
                self.chunk_ref.index_block(BlockPos::new(x + 1, y, z))
            }
            Face::Left => {
                self.chunk_ref.index_block(BlockPos::new(x.checked_sub(1)?, y, z))
            }
            Face::Up => {
                self.chunk_ref.index_block(BlockPos::new(x, y + 1, z))
            }
            Face::Down => {
                self.chunk_ref.index_block(BlockPos::new(x, y.checked_sub(1)?, z))
            }
        }
    }
}

#[derive(Debug)]
pub struct Chunk<const L: usize, const H: usize> {
    blocks: [[[Block; L]; L]; H]
}

impl<const L: usize, const H: usize> Chunk<L, H> {
    pub fn new() -> Self {
        Self {
            blocks: [[[Block::Air; L]; L]; H]
        }
    }
    
    fn index_block<'a>(
        &'a self,
        block_pos @ BlockPos { x, y, z }: BlockPos
    ) -> Option<BlockRef<'a, L, H>> {
        self.blocks.get(y)
            .and_then(|bs| bs.get(z)
                .and_then(|bs| bs.get(x)))
            .map(|block_ref| {
                BlockRef {
                    block_ref,
                    chunk_ref: self,
                    block_pos
                }
            })
    }
    
    fn index_block_mut(&mut self, BlockPos { x, y, z }: BlockPos) -> Option<&mut Block> {
        self.blocks.get_mut(y)
            .and_then(|bs| bs.get_mut(z)
                .and_then(|bs| bs.get_mut(x)))
    }
    
    pub fn place_block(
        &mut self,
        block_pos: BlockPos,
        block: Block
    ) -> Option<()> {
        *self.index_block_mut(block_pos)? = block;

        Some(())
    }

    pub fn iter<'a>(&'a self) -> ChunkIter<'a, L, H> {
        ChunkIter {
            chunk: self,
            block_pos: BlockPos::new(0, 0, 0)
        }
    }
}

pub struct ChunkIter<'a, const L: usize, const H: usize> {
    chunk: &'a Chunk<L, H>,
    block_pos: BlockPos
}

impl<'a, const L: usize, const H: usize> ChunkIter<'a, L, H> {
    fn advance_index(&mut self) -> bool {
        let BlockPos { ref mut x, ref mut y, ref mut z } = self.block_pos;
        if *x == L - 1 {
            *x = 0;
            if *z == L - 1 {
                *z = 0;
                if *y == H {
                    false
                } else {
                    *y += 1;
                    true
                }
            } else {
                *z += 1;
                true
            }
        } else {
            *x += 1;
            true
        }
    }
}

impl<'a, const L: usize, const H: usize> Iterator for ChunkIter<'a, L, H> {
    type Item = BlockRef<'a, L, H>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.chunk.index_block(self.block_pos);

        if !self.advance_index() {
            return None;
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn chunk_iteration_and_access() {
        let mut chunk: Chunk<2, 2> = Chunk::new();
        chunk.place_block(BlockPos::new(0, 0, 0), Block::Id(1)).unwrap();
        chunk.place_block(BlockPos::new(1, 0, 0), Block::Id(2)).unwrap();
        chunk.place_block(BlockPos::new(0, 0, 1), Block::Id(3)).unwrap();
        chunk.place_block(BlockPos::new(1, 0, 1), Block::Id(4)).unwrap();
        chunk.place_block(BlockPos::new(0, 1, 0), Block::Id(5)).unwrap();
        chunk.place_block(BlockPos::new(1, 1, 0), Block::Id(6)).unwrap();
        chunk.place_block(BlockPos::new(0, 1, 1), Block::Id(7)).unwrap();
        chunk.place_block(BlockPos::new(1, 1, 1), Block::Id(8)).unwrap();

        let b = chunk.index_block(BlockPos::new(0, 0, 0)).unwrap();
        assert_eq!(b.neighbor(Face::Front).map(|b| b.block_ref), Some(&Block::Id(2)));
        assert_eq!(b.neighbor(Face::Right).map(|b| b.block_ref), Some(&Block::Id(3)));
        assert_eq!(b.neighbor(Face::Back).map(|b| b.block_ref), None);
        assert_eq!(b.neighbor(Face::Left).map(|b| b.block_ref), None);
        assert_eq!(
            b.neighbor(Face::Right)
                .and_then(|b| b.neighbor(Face::Up))
                    .map(|b| b.block_ref), 
            Some(&Block::Id(7))
        );
    }

    #[test]
    fn quad_mesh() {
        let mut chunk: Chunk<2, 2> = Chunk::new();
        chunk.place_block(BlockPos::new(0, 0, 0), Block::Dirt).unwrap();

        let mut mesher = VoxelMesh::new();
        mesher.serialize_chunk(&chunk);

        assert_eq!(
            mesher.faces,
            vec![
                Face::Front, 
                Face::Back,  
                Face::Up,    
                Face::Down,  
                Face::Left,  
                Face::Right, 
            ]
        );

        assert_eq!(
            mesher.mesh(),
            {
                let mut builder = MeshBuilder::new();
                builder.push(Mesh::FRONT_FACE, BlockPos::new(0, 0, 0));
                builder.push(Mesh::BACK_FACE, BlockPos::new(0, 0, 0));
                builder.push(Mesh::UP_FACE, BlockPos::new(0, 0, 0));
                builder.push(Mesh::DOWN_FACE, BlockPos::new(0, 0, 0));
                builder.push(Mesh::LEFT_FACE, BlockPos::new(0, 0, 0));
                builder.push(Mesh::RIGHT_FACE, BlockPos::new(0, 0, 0));
                builder.build()
            }
        );
    }
}