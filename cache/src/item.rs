use crate::cell::{Bytes, BytesRef, Timestamp};

const BLOCK_SIZE: usize = 4096;
const READ_UNIT_SIZE: usize = 1024;
const BLOCK_UNIT_NUM: usize = BLOCK_SIZE / READ_UNIT_SIZE;

struct Block {
    // todo: type param
    block: [u8; BLOCK_SIZE],
    used: usize,
}

impl Block {
    pub fn get(&self, pos: usize) -> Bytes {
        self.block[pos * READ_UNIT_SIZE..(pos + 1) * READ_UNIT_SIZE].to_vec()
    }

    pub fn put(&mut self, bytes: Bytes) {
        todo!()
    }
}

#[derive(Default)]
pub struct Item {
    blocks: Vec<Block>,
}

impl Item {
    pub fn get(&self, pos: usize) -> Bytes {
        let (i, pos) = self.convert_pos(pos);
        self.blocks[i].get(pos)
    }

    pub fn put(&mut self, bytes: Bytes) {
        todo!()
    }

    /// Return index of / inside block
    #[inline]
    fn convert_pos(&self, pos: usize) -> (usize, usize) {
        let index_of_block = (pos / BLOCK_UNIT_NUM) % self.blocks.len();
        let index_inside_block = pos % BLOCK_UNIT_NUM;

        (index_of_block, index_inside_block)
    }
}
