use crate::chunk::{Block, BlockPos, ChunkPos, Chunk};

pub struct World {
    chunks: Vec<Chunk<16, 16>>,
    scheduled_chunks: Vec<Chunk<16, 16>>,
    to_update_chunks: Vec<Chunk<16, 16>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            scheduled_chunks: {
                const RD: i32 = 8;
                let mut v = Vec::new();
                for x in -RD..RD {
                    for z in -RD..RD {
                        v.push({
                            let mut chunk = Chunk::new(ChunkPos::new(x, z));
                            for x in 0..16 {
                                for y in 0..x {
                                    for z in 0..16 {
                                        chunk.place_block(BlockPos::new(x, y, z), Block::Dirt);
                                    }
                                }
                            }
                            chunk
                        });
                    }
                }
                v
            },
            to_update_chunks: Vec::new()
        }
    }

    pub fn to_update_chunks<'a>(
        &'a mut self
    ) -> impl Iterator<Item = &'a Chunk<16, 16>> {
        let l = self.chunks.len();
        self.chunks.append(&mut self.to_update_chunks);

        self.chunks.iter().skip(l)
    }

    pub fn scheduled_chunks<'a>(
        &'a mut self
    ) -> impl Iterator<Item = &'a Chunk<16, 16>> {
        let l = self.chunks.len();
        self.chunks.append(&mut self.scheduled_chunks);

        self.chunks.iter().skip(l)
    }

    pub fn chunks<'a>(&'a self) -> impl Iterator<Item = &'a Chunk<16, 16>> {
        self.chunks.iter()
    }
}

