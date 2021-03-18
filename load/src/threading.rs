use cache::{Bytes, Cache, Id};

const SHARD_NUM: usize = 128;

pub struct ThreadingLoad {
    shards: Vec<Cache>,
}

impl ThreadingLoad {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut shards = Vec::with_capacity(SHARD_NUM);
        shards.resize_with(SHARD_NUM, Default::default);
        Self { shards }
    }

    pub fn append(&self, id: Id, bytes: Bytes) {
        let bytes = Self::calculation(bytes);
        self.shards[self.shard_id(id)].append(id, bytes);
    }

    pub fn get(&self, id: Id, size: usize) -> Option<Bytes> {
        self.shards[self.shard_id(id)]
            .get(id, size)
            .map(Self::calculation)
    }

    #[inline]
    fn shard_id(&self, id: Id) -> usize {
        id % SHARD_NUM
    }

    #[inline]
    fn calculation(mut bytes: Bytes) -> Bytes {
        let mut sum: u8 = 0;
        bytes.iter().for_each(|x| sum = sum.wrapping_add(*x));
        bytes[0] += sum;
        bytes
    }
}
