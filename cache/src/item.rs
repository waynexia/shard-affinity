use crate::cell::{Bytes, BytesRef};
use rand::random;

const BLOCK_SIZE: usize = 4096;

struct Block {
    // todo: type param
    block: [u8; BLOCK_SIZE],
    used: usize,
}

impl Block {
    pub fn new() -> Self {
        Self {
            block: [0; BLOCK_SIZE],
            used: 0,
        }
    }

    pub fn get(&self, size: usize) -> BytesRef {
        // todo: add calculation
        &self.block[0..size]
    }

    /// Return un-capacity size
    pub fn put(&mut self, bytes: BytesRef) -> usize {
        let bytes_len = bytes.len();
        if BLOCK_SIZE - self.used >= bytes_len {
            self.block[self.used..self.used + bytes_len].copy_from_slice(bytes);
            self.used += bytes_len;
            0
        } else {
            let remaining = BLOCK_SIZE - self.used;
            self.block[self.used..].copy_from_slice(&bytes[..remaining]);
            self.used = BLOCK_SIZE;
            bytes_len - remaining
        }
    }
}

#[derive(Default)]
pub struct Item {
    blocks: Vec<Block>,
}

impl Item {
    pub fn get(&self, size: usize) -> Bytes {
        let block_num = self.blocks.len();
        let mut result = Vec::with_capacity(size);

        while result.len() < size {
            let index = random::<usize>() % block_num;
            result.extend_from_slice(self.blocks[index].get(size - result.len()));
        }

        result
    }

    pub fn put(&mut self, bytes: Bytes) {
        let mut cursor = 0;
        let mut remaining = self.blocks.last_mut().unwrap().put(&bytes[cursor..]);
        while remaining != 0 {
            self.blocks.push(Block::new());
            cursor += remaining;
            remaining = self.blocks.last_mut().unwrap().put(&bytes[cursor..]);
        }
    }
}
